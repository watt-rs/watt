/// Imports
use crate::analyze::{
    env::{EnvironmentType, EnvironmentsStack},
    errors::AnalyzeError,
    rc_ptr::RcPtr,
    res::Res,
    typ::{CustomType, Enum, EnumVariant, Function, Module, PreludeType, Typ, Type, WithPublicity},
    warnings::AnalyzeWarning,
};
use ecow::EcoString;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::{address::Address, bail, warn};
use oil_ir::ir::{
    IrBinaryOp, IrBlock, IrDeclaration, IrEnumConstructor, IrExpression, IrFunction, IrModule,
    IrParameter, IrStatement, IrUnaryOp, IrVariable,
};
use std::{cell::RefCell, collections::HashMap};

/// Call result
pub enum CallResult {
    FromFunction(Typ, RcPtr<Function>),
    FromType(Typ),
    FromEnum(Typ),
    FromDyn,
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    module: &'pkg IrModule,
    environments_stack: EnvironmentsStack,
    custom_types: HashMap<EcoString, WithPublicity<CustomType>>,
    modules: HashMap<EcoString, &'pkg Module>,
}

/// Implementation
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Creates new module analyzer
    pub fn new(modules: HashMap<EcoString, &'pkg Module>, module: &'pkg IrModule) -> Self {
        Self {
            module,
            environments_stack: EnvironmentsStack::new(),
            custom_types: HashMap::new(),
            modules,
        }
    }

    /// Infers binary
    fn infer_binary(
        &self,
        location: Address,
        op: IrBinaryOp,
        left: IrExpression,
        right: IrExpression,
    ) -> Typ {
        // Inferred left and right `Typ`
        let inferred_left = self.infer_expr(left);
        let inferred_right = self.infer_expr(right);

        // Left and right `PreludeType`
        let left_typ;
        let right_typ;

        // Checking, both types are prelude
        match &inferred_left {
            Typ::Prelude(l) => {
                left_typ = l;
                match &inferred_right {
                    Typ::Prelude(r) => {
                        right_typ = r;
                    }
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op: op.into()
                    }),
                }
            }
            _ => bail!(AnalyzeError::InvalidBinaryOp {
                src: self.module.source.clone(),
                span: (location.span.start..location.span.end).into(),
                a: inferred_left,
                b: inferred_right,
                op: op.into()
            }),
        }

        // Matching operator
        match op {
            // Arithmetical
            IrBinaryOp::Add
            | IrBinaryOp::Mul
            | IrBinaryOp::Sub
            | IrBinaryOp::Div
            | IrBinaryOp::BitwiseAnd
            | IrBinaryOp::BitwiseOr
            | IrBinaryOp::Mod => {
                // Checking prelude types
                match left_typ {
                    PreludeType::Int => match right_typ {
                        PreludeType::Int => Typ::Prelude(PreludeType::Int),
                        PreludeType::Float => Typ::Prelude(PreludeType::Float),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    PreludeType::Float => match right_typ {
                        PreludeType::Int => Typ::Prelude(PreludeType::Float),
                        PreludeType::Float => Typ::Prelude(PreludeType::Float),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op
                    }),
                }
            }
            // Logical
            IrBinaryOp::Xor | IrBinaryOp::And | IrBinaryOp::Or => {
                // Checking prelude types
                match left_typ {
                    PreludeType::Bool => match right_typ {
                        PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op
                    }),
                }
            }
            // Compare
            IrBinaryOp::Le | IrBinaryOp::Lt | IrBinaryOp::Ge | IrBinaryOp::Gt => {
                // Checking prelude types
                match left_typ {
                    PreludeType::Int | PreludeType::Float => match right_typ {
                        PreludeType::Int | PreludeType::Float => Typ::Prelude(PreludeType::Bool),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op
                    }),
                }
            }
            // Equality
            IrBinaryOp::Eq | IrBinaryOp::Neq => Typ::Prelude(PreludeType::Bool),
        }
    }

    /// Infers unary
    fn infer_unary(&self, location: Address, op: IrUnaryOp, value: IrExpression) -> Typ {
        // Inferred value `Typ`
        let inferred_value = self.infer_expr(value);

        // Value `PreludeType`
        let value_typ;

        // Checking type is prelude
        match &inferred_value {
            Typ::Prelude(t) => {
                value_typ = t;
            }
            _ => bail!(AnalyzeError::InvalidUnaryOp {
                src: self.module.source.clone(),
                span: (location.span.start..location.span.end).into(),
                t: inferred_value,
                op
            }),
        }

        // Matching operator
        match op {
            IrUnaryOp::Negate => match value_typ {
                PreludeType::Int => Typ::Prelude(PreludeType::Int),
                PreludeType::Float => Typ::Prelude(PreludeType::Float),
                _ => bail!(AnalyzeError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: (location.span.start..location.span.end).into(),
                    t: inferred_value,
                    op
                }),
            },
            IrUnaryOp::Bang => match value_typ {
                PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                _ => bail!(AnalyzeError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: (location.span.start..location.span.end).into(),
                    t: inferred_value,
                    op
                }),
            },
        }
    }

    /// Infers get
    fn infer_get(&self, location: Address, name: EcoString) -> Res {
        // Local variable
        if self.environments_stack.exists(&name) {
            Res::Value(
                self.environments_stack
                    .lookup(&self.module.source, &location, &name),
            )
        }
        // Module
        else if self.modules.contains_key(&name) {
            Res::Module(name)
        }
        // Type or could not resolve
        else {
            // Checking type existence
            match self.custom_types.get(&name) {
                Some(t) => Res::Custom(t.value.clone()),
                None => bail!(AnalyzeError::CouldNotResolve {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    name
                }),
            }
        }
    }

    /// Infers module field access
    fn infer_module_field_access(
        &self,
        field_location: Address,
        field_module: EcoString,
        field_name: EcoString,
    ) -> Res {
        // Getting module
        match self.modules.get(&field_module) {
            // Getting module
            Some(module) => match module.environment.get(&field_name) {
                // If environment field exists,
                // checking it's publicity
                Some(var_typ) => match var_typ.publicity {
                    // If environment field is public, we resolved field
                    Publicity::Public => Res::Value(var_typ.value.clone()),
                    // Else, checking custom type field exists
                    _ => match module.custom_types.get(&field_name) {
                        // If type exists
                        Some(custom) => {
                            // Checking its publicity
                            match custom.publicity {
                                // If type is public
                                Publicity::Public => Res::Custom(custom.value.clone()),
                                // If type is private, raising `both module fields is private`
                                _ => bail!(AnalyzeError::BothModuleFieldsIsPrivate {
                                    src: self.module.source.clone(),
                                    span: field_location.span.into(),
                                    name: field_name
                                }),
                            }
                        }
                        // Else, raising `module field is private`
                        None => bail!(AnalyzeError::ModuleFieldIsPrivate {
                            src: self.module.source.clone(),
                            span: field_location.span.into(),
                            name: field_name
                        }),
                    },
                },
                // If no environment field found, finding type field
                _ => match module.custom_types.get(&field_name) {
                    // If type exists
                    Some(custom) => {
                        // Checking its publicity
                        match custom.publicity {
                            // If type is public
                            Publicity::Public => Res::Custom(custom.value.clone()),
                            // If type is private, raising `type is private`
                            _ => bail!(AnalyzeError::TypeIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                t: custom.value.clone()
                            }),
                        }
                    }
                    // Else, raising `module field is private`
                    None => bail!(AnalyzeError::ModuleFieldIsPrivate {
                        src: self.module.source.clone(),
                        span: field_location.span.into(),
                        name: field_name
                    }),
                },
            },
            // If module is not defined
            None => bail!(AnalyzeError::ModuleIsNotDefined { m: field_module }),
        }
    }

    /// Infers enum field access
    fn infer_enum_field_access(
        &self,
        field_location: Address,
        en: RcPtr<Enum>,
        field_name: EcoString,
    ) -> Res {
        // Finding variant
        match en.variants.iter().find(|var| var.name == field_name) {
            Some(variant) => Res::Variant(en.clone(), variant.clone()),
            None => bail!(AnalyzeError::EnumVariantIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                e: en.name.clone(),
                variant: field_name
            }),
        }
    }

    /// Infers type field access
    fn infer_type_field_access(
        &self,
        field_location: Address,
        ty: RcPtr<RefCell<Type>>,
        field_name: EcoString,
    ) -> Res {
        // Finding field
        let borrowed = ty.borrow();
        match borrowed.env.get(&field_name) {
            Some(typ) => match typ.publicity {
                // Checking publicity
                Publicity::Public => Res::Value(typ.value.clone()),
                // Checking environments stack contains type
                _ => match self.environments_stack.contains_type() {
                    // If type is same
                    Some(t) => {
                        if *t == ty {
                            Res::Value(typ.value.clone())
                        } else {
                            bail!(AnalyzeError::FieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                t: borrowed.name.clone(),
                                field: field_name
                            });
                        }
                    }
                    // Else
                    None => bail!(AnalyzeError::FieldIsPrivate {
                        src: self.module.source.clone(),
                        span: field_location.span.into(),
                        t: borrowed.name.clone(),
                        field: field_name
                    }),
                },
            },
            None => bail!(AnalyzeError::FieldIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                t: borrowed.name.clone(),
                field: field_name
            }),
        }
    }

    /// Infers access
    fn infer_field_access(
        &self,
        field_location: Address,
        container: IrExpression,
        field_name: EcoString,
    ) -> Res {
        // Inferring container
        let container_inferred = self.infer_resolution(container);
        match &container_inferred {
            // Module field access
            Res::Module(name) => {
                self.infer_module_field_access(field_location, name.clone(), field_name)
            }
            // Enum field access
            Res::Custom(custom) => match custom {
                CustomType::Enum(en) => {
                    self.infer_enum_field_access(field_location, en.clone(), field_name)
                }
                _ => bail!(AnalyzeError::CouldNotResolveFieldsIn {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    res: container_inferred
                }),
            },
            // Type field access
            Res::Value(typ) => match typ {
                // Custom Type
                Typ::Custom(t) => {
                    self.infer_type_field_access(field_location, t.clone(), field_name)
                }
                // Dyn
                Typ::Dyn => {
                    // Returning `dyn` like field type,
                    // but emitting warning
                    warn!(AnalyzeWarning::AccessOfDynField {
                        src: self.module.source.clone(),
                        span: field_location.span.into()
                    });
                    Res::Value(Typ::Dyn)
                }
                // Else
                _ => bail!(AnalyzeError::CouldNotResolveFieldsIn {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    res: container_inferred
                }),
            },
            // Else
            _ => bail!(AnalyzeError::CouldNotResolveFieldsIn {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                res: container_inferred
            }),
        }
    }

    /// Infers call
    fn infer_call(
        &self,
        location: Address,
        what: IrExpression,
        args: Vec<IrExpression>,
    ) -> CallResult {
        let function = self.infer_resolution(what);
        let args = args
            .iter()
            .map(|a| self.infer_expr(a.clone()))
            .collect::<Vec<Typ>>();
        match &function {
            // Custom type
            Res::Custom(t) => match t {
                CustomType::Type(ty) => {
                    let borrowed = ty.borrow();
                    if borrowed.params != args {
                        bail!(AnalyzeError::InvalidArgs {
                            src: self.module.source.clone(),
                            params_span: borrowed.location.span.clone().into(),
                            span: location.span.into()
                        })
                    } else {
                        return CallResult::FromType(Typ::Custom(ty.clone()));
                    }
                }
                _ => bail!(AnalyzeError::CouldNotCall {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    res: function
                }),
            },
            // Value
            Res::Value(t) => match t {
                // Function
                Typ::Function(f) => {
                    if f.params != args {
                        bail!(AnalyzeError::InvalidArgs {
                            src: self.module.source.clone(),
                            params_span: f.location.span.clone().into(),
                            span: location.span.into()
                        })
                    } else {
                        return CallResult::FromFunction(f.ret.clone(), f.clone());
                    }
                }
                // Dyn
                Typ::Dyn => {
                    // Returning `dyn` call result,
                    // but emitting warning
                    warn!(AnalyzeWarning::CallOfDyn {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    });
                    CallResult::FromDyn
                }
                // Else
                _ => bail!(AnalyzeError::CouldNotCall {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    res: function
                }),
            },
            // Variant
            Res::Variant(en, variant) => {
                if variant.params.values().cloned().collect::<Vec<Typ>>() != args {
                    bail!(AnalyzeError::InvalidArgs {
                        src: self.module.source.clone(),
                        params_span: variant.location.span.clone().into(),
                        span: location.span.into()
                    })
                } else {
                    return CallResult::FromEnum(Typ::Enum(en.clone()));
                }
            }
            _ => bail!(AnalyzeError::CouldNotCall {
                src: self.module.source.clone(),
                span: location.span.into(),
                res: function
            }),
        }
    }

    /// Infers type annotation
    fn infer_type_annotation(&self, path: TypePath) -> Typ {
        match path {
            TypePath::Local { location, name } => match name.as_str() {
                "int" => Typ::Prelude(PreludeType::Int),
                "float" => Typ::Prelude(PreludeType::Float),
                "bool" => Typ::Prelude(PreludeType::Bool),
                "string" => Typ::Prelude(PreludeType::String),
                "dyn" => Typ::Dyn,
                _ => match self.custom_types.get(&name) {
                    Some(t) => match &t.value {
                        CustomType::Enum(en) => Typ::Enum(en.clone()),
                        CustomType::Type(ty) => Typ::Custom(ty.clone()),
                    },
                    None => bail!(AnalyzeError::TypeIsNotDefined {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        t: name
                    }),
                },
            },
            TypePath::Module {
                location,
                module,
                name,
            } => {
                let m = match self.modules.get(&module) {
                    Some(m) => m,
                    None => bail!(AnalyzeError::ModuleIsNotDefined { m: module }),
                };
                let typ = match m.custom_types.get(&name) {
                    Some(t) => {
                        if t.publicity != Publicity::Private {
                            match &t.value {
                                CustomType::Enum(en) => Typ::Enum(en.clone()),
                                CustomType::Type(ty) => Typ::Custom(ty.clone()),
                            }
                        } else {
                            bail!(AnalyzeError::TypeIsPrivate {
                                src: self.module.source.clone(),
                                span: location.span.into(),
                                t: t.value.clone()
                            })
                        }
                    }
                    None => bail!(AnalyzeError::TypeIsNotDefined {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        t: format!("{}.{}", module, name).into()
                    }),
                };
                typ
            }
        }
    }

    /// Infers resolution
    fn infer_resolution(&self, expr: IrExpression) -> Res {
        match expr {
            IrExpression::Get { location, name } => self.infer_get(location, name),
            IrExpression::FieldAccess {
                location,
                container,
                name,
            } => self.infer_field_access(location, *container, name),
            IrExpression::Call {
                location,
                what,
                args,
            } => match self.infer_call(location.clone(), *what, args) {
                CallResult::FromFunction(typ, function) => {
                    if typ == Typ::Void {
                        bail!(AnalyzeError::CallExprReturnTypeIsVoid {
                            fn_src: function.source.clone(),
                            definition_span: function.location.span.clone().into(),
                            call_src: self.module.source.clone(),
                            span: location.span.into()
                        })
                    } else {
                        Res::Value(typ)
                    }
                }
                CallResult::FromType(t) => Res::Value(t),
                CallResult::FromEnum(e) => Res::Value(e),
                CallResult::FromDyn => Res::Value(Typ::Dyn),
            },
            expr => bail!(AnalyzeError::UnexpectedExprInResolution {
                expr: format!("{:?}", expr).into()
            }),
        }
    }

    /// Infers expression
    fn infer_expr(&self, expr: IrExpression) -> Typ {
        match expr {
            IrExpression::Float { .. } => Typ::Prelude(PreludeType::Float),
            IrExpression::Int { .. } => Typ::Prelude(PreludeType::Int),
            IrExpression::String { .. } => Typ::Prelude(PreludeType::String),
            IrExpression::Bool { .. } => Typ::Prelude(PreludeType::Bool),
            IrExpression::Bin {
                location,
                left,
                right,
                op,
            } => self.infer_binary(location, op, *left, *right),
            IrExpression::Unary {
                location,
                value,
                op,
            } => self.infer_unary(location, op, *value),
            IrExpression::Get { location, name } => self
                .infer_get(location.clone(), name)
                .unwrap_typ(&self.module.source, &location),
            IrExpression::FieldAccess {
                location,
                container,
                name,
            } => self
                .infer_field_access(location.clone(), *container, name)
                .unwrap_typ(&self.module.source, &location),
            IrExpression::Call {
                location,
                what,
                args,
            } => match self.infer_call(location.clone(), *what, args) {
                CallResult::FromFunction(typ, function) => {
                    if typ == Typ::Void {
                        bail!(AnalyzeError::CallExprReturnTypeIsVoid {
                            fn_src: function.source.clone(),
                            definition_span: function.location.span.clone().into(),
                            call_src: self.module.source.clone(),
                            span: location.span.into()
                        })
                    } else {
                        typ
                    }
                }
                CallResult::FromType(t) => t,
                CallResult::FromEnum(e) => e,
                CallResult::FromDyn => Typ::Dyn,
            },
            IrExpression::Range { .. } => todo!(),
        }
    }

    /// Analyzes block
    fn analyze_block(&mut self, block: IrBlock) {
        for statement in block.nodes {
            self.analyze_statement(statement);
        }
    }

    /// Analyzes define
    fn analyze_define(
        &mut self,
        location: Address,
        name: EcoString,
        value: IrExpression,
        typ: Option<TypePath>,
    ) {
        let inferred_value = self.infer_expr(value);
        match typ {
            Some(annotated_path) => {
                let annotated = self.infer_type_annotation(annotated_path);
                if inferred_value == annotated {
                    // defining by annotated type, because:
                    //
                    // 1. we can get
                    // non-dyn value, but annotation will be `dyn`
                    //
                    // 2. we can get
                    // dyn value, but annotation will not be `dyn`
                    //
                    self.environments_stack
                        .define(&self.module.source, &location, &name, annotated)
                } else {
                    bail!(AnalyzeError::MissmatchedTypeAnnotation {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        expected: annotated,
                        got: inferred_value
                    })
                }
            }
            None => self.environments_stack.define(
                &self.module.source,
                &location,
                &name,
                inferred_value,
            ),
        }
    }

    /// Analyzes assignment
    fn analyze_assignment(&mut self, location: Address, what: IrExpression, value: IrExpression) {
        let inferred_what = self.infer_expr(what);
        let inferred_value = self.infer_expr(value);
        if inferred_what != inferred_value {
            bail!(AnalyzeError::TypesMissmatch {
                src: self.module.source.clone(),
                span: location.span.into(),
                expected: inferred_what,
                got: inferred_value
            })
        }
    }

    /// Analyzes funciton
    fn analyze_function(
        &mut self,
        location: Address,
        name: EcoString,
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_annotation(t));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params
                .clone()
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };
        self.environments_stack.define(
            &self.module.source.clone(),
            &location,
            &name,
            Typ::Function(RcPtr::new(function)),
        );

        // pushing new scope
        self.environments_stack
            .push(EnvironmentType::Function(ret.clone()));

        // defining params in new scope
        params.iter().for_each(|p| {
            self.environments_stack
                .define(&self.module.source, &location, p.0, p.1.clone())
        });

        // inferring body
        self.analyze_block(body);
        self.environments_stack.pop();
    }

    /// Analyzes statement
    fn analyze_statement(&mut self, statement: IrStatement) {
        match statement {
            IrStatement::If {
                location,
                logical,
                body,
                elseif,
            } => {
                // pushing env
                self.environments_stack.push(EnvironmentType::Conditional);
                // inferring logical
                let inferred_logical = self.infer_expr(logical);
                match inferred_logical {
                    Typ::Prelude(prelude) => match prelude {
                        PreludeType::Bool => {}
                        _ => bail!(AnalyzeError::ExpectedLogicalInIf {
                            src: self.module.source.clone(),
                            span: location.span.into()
                        }),
                    },
                    _ => bail!(AnalyzeError::ExpectedLogicalInIf {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    }),
                }
                // analyzing block
                self.analyze_block(body);
                // popping env
                self.environments_stack.pop();
                // analyzing elseif
                match elseif {
                    Some(elseif) => self.analyze_statement(*elseif),
                    None => {}
                }
            }
            IrStatement::While {
                location,
                logical,
                body,
            } => {
                // pushing env
                self.environments_stack.push(EnvironmentType::Loop);
                // inferring logical
                let inferred_logical = self.infer_expr(logical);
                match inferred_logical {
                    Typ::Prelude(prelude) => match prelude {
                        PreludeType::Bool => {}
                        _ => bail!(AnalyzeError::ExpectedLogicalInWhile {
                            src: self.module.source.clone(),
                            span: location.span.into()
                        }),
                    },
                    _ => bail!(AnalyzeError::ExpectedLogicalInWhile {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    }),
                }
                // analyzing block
                self.analyze_block(body);
                // popping env
                self.environments_stack.pop();
            }
            IrStatement::Define {
                location,
                name,
                value,
                typ,
            } => self.analyze_define(location, name, value, typ),
            IrStatement::Assign {
                location,
                what,
                value,
            } => self.analyze_assignment(location, *what, *value),
            IrStatement::Call {
                location,
                what,
                args,
            } => {
                self.infer_call(location, *what, args);
            }
            IrStatement::Fn {
                location,
                name,
                params,
                body,
                typ,
            } => self.analyze_function(location, name, params, body, typ),
            IrStatement::Break { location } => {
                if !self.environments_stack.contains_env(EnvironmentType::Loop) {
                    bail!(AnalyzeError::BreakWithoutLoop {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    })
                }
            }
            IrStatement::Continue { location } => {
                if !self.environments_stack.contains_env(EnvironmentType::Loop) {
                    bail!(AnalyzeError::ContinueWithoutLoop {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    })
                }
            }
            IrStatement::Return { location, value } => {
                let inferred_value = self.infer_expr(value);
                match self.environments_stack.contains_function() {
                    Some(ret_type) => {
                        if &inferred_value != ret_type {
                            bail!(AnalyzeError::WrongReturnType {
                                src: self.module.source.clone(),
                                span: location.span.into(),
                                expected: ret_type.clone(),
                                got: inferred_value
                            })
                        }
                    }
                    None => bail!(AnalyzeError::ReturnWithoutFunction {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    }),
                }
            }
            IrStatement::For { .. } => todo!(),
        }
    }

    /// Analyzes method
    fn analyze_method(
        &mut self,
        location: Address,
        name: EcoString,
        type_: RcPtr<RefCell<Type>>,
        publicity: Publicity,
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_annotation(t));
        self.environments_stack
            .push(EnvironmentType::Function(ret.clone()));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        params.iter().for_each(|p| {
            self.environments_stack
                .define(&self.module.source, &location, p.0, p.1.clone())
        });

        self.environments_stack.define(
            &self.module.source,
            &location,
            &"self".into(),
            Typ::Custom(type_.clone()),
        );

        // inferring body
        self.analyze_block(body);
        self.environments_stack.pop();

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params.into_iter().map(|(_, v)| v).collect::<Vec<Typ>>(),
            ret,
        };

        // defining function, if not already defined
        if let Some(_) = type_.borrow().env.get(&name).cloned() {
            bail!(AnalyzeError::MethodIsAlreadyDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                m: name
            })
        } else {
            type_.borrow_mut().env.insert(
                name,
                WithPublicity {
                    publicity,
                    value: Typ::Function(RcPtr::new(function)),
                },
            );
        }
    }

    /// Analyzes type
    fn analyze_type(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<IrParameter>,
        fields: Vec<IrVariable>,
        functions: Vec<IrFunction>,
    ) {
        // inferred params
        let inferred_params = params
            .into_iter()
            .map(|p| (p.name, (p.location, self.infer_type_annotation(p.typ))))
            .collect::<HashMap<EcoString, (Address, Typ)>>();

        // construction type
        let type_ = RcPtr::new(RefCell::new(Type {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: inferred_params.iter().map(|p| p.1.1.clone()).collect(),
            env: HashMap::new(),
        }));

        // params env start
        self.environments_stack
            .push(EnvironmentType::ConstructorParams);

        // params
        inferred_params.into_iter().for_each(|p| {
            self.environments_stack
                .define(&self.module.source, &p.1.0, &p.0, p.1.1);
        });

        // fields env start
        self.environments_stack.push(EnvironmentType::Fields);

        // fields
        fields.clone().into_iter().for_each(|f| {
            self.analyze_define(f.location, f.name, f.value, f.typ);
        });

        // fields env end
        let analyzed_fields = match self.environments_stack.pop() {
            Some(fields) => fields.1,
            None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
        };

        // params env end
        self.environments_stack.pop();

        // adding fields to type env
        let mut borrowed = type_.borrow_mut();
        fields.into_iter().for_each(|f| {
            borrowed.env.insert(
                f.name.clone(),
                WithPublicity {
                    publicity: f.publicity,
                    value: analyzed_fields.get(&f.name).unwrap().clone(),
                },
            );
        });
        drop(borrowed);

        // type env start
        self.environments_stack
            .push(EnvironmentType::Type(type_.clone()));

        // adding functions
        functions.into_iter().for_each(|f| {
            self.analyze_method(
                f.location,
                f.name,
                type_.clone(),
                f.publicity,
                f.params,
                f.body,
                f.typ,
            );
        });

        // type env end
        self.environments_stack.pop();

        // defining type, if not already defined
        match self.custom_types.get(&name) {
            Some(_) => bail!(AnalyzeError::TypeIsAlreadyDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: name
            }),
            None => {
                self.custom_types.insert(
                    name,
                    WithPublicity {
                        publicity,
                        value: CustomType::Type(type_),
                    },
                );
            }
        }
    }

    /// Analyzes enum
    fn analyze_enum(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        variants: Vec<IrEnumConstructor>,
    ) {
        // inferred variants
        let inferred_variants = variants
            .into_iter()
            .map(|v| EnumVariant {
                location: v.location,
                name: v.name,
                params: v
                    .params
                    .into_iter()
                    .map(|param| (param.name, self.infer_type_annotation(param.typ)))
                    .collect(),
            })
            .collect::<Vec<EnumVariant>>();

        // construction enum
        let enum_ = RcPtr::new(Enum {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            variants: inferred_variants,
        });

        // defining enum
        match self.custom_types.get(&name) {
            Some(_) => bail!(AnalyzeError::TypeIsAlreadyDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: name
            }),
            None => {
                self.custom_types.insert(
                    name,
                    WithPublicity {
                        publicity,
                        value: CustomType::Enum(enum_),
                    },
                );
            }
        }
    }

    /// Analyzes declaration
    pub fn analyze_declaration(&mut self, declaration: IrDeclaration) {
        match declaration {
            IrDeclaration::Function(ir_function) => self.analyze_function(
                ir_function.location,
                ir_function.name,
                ir_function.params,
                ir_function.body,
                ir_function.typ,
            ),
            IrDeclaration::Variable(ir_variable) => self.analyze_define(
                ir_variable.location,
                ir_variable.name,
                ir_variable.value,
                ir_variable.typ,
            ),
            IrDeclaration::Type(ir_type) => self.analyze_type(
                ir_type.location,
                ir_type.name,
                ir_type.publicity,
                ir_type.constructor,
                ir_type.fields,
                ir_type.functions,
            ),
            IrDeclaration::Enum(ir_enum) => self.analyze_enum(
                ir_enum.location,
                ir_enum.name,
                ir_enum.publicity,
                ir_enum.variants,
            ),
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) {
        self.environments_stack.push(EnvironmentType::Module);
        for definition in self.module.clone().definitions {
            self.analyze_declaration(definition)
        }
    }
}
