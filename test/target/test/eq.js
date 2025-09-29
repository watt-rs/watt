function do(f) {
    f(15, 13);
}

function eq() {
    do(function (a, b) {
        return a + b;
    });
}
