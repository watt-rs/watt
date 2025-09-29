function sum(a, b) {
    return a + b;
}

function do(function) {
    function(15, 13);
}

function eq() {
    do(sum);
}
