import("./rsmodel.js").then(rsmodel => {
    let widgets = [];
    [
        "c0", "c1", "c2",
        "c3", "c4", "c5",
        "c6", "c7", "c8"
    ].forEach(function(el, i) {
        widgets.push(document.getElementById(el))
    });

    let whoseTurn = function() {
        let view = rsmodel.viewBoard();
        if (view.outcome != null) {
            return "nobody";
        }
        if (view.playerTurn == 0) {
            if (document.getElementById("p0robot").checked) { return "robot"; }
            return "player";
        }

        if (view.playerTurn == 1) {
            if (document.getElementById("p1robot").checked) { return "robot"; }
            return "player";
        } 
    }

    let setUp = function() { 
        rsmodel.start();
        widgets.forEach((widg, i) => {
            widg.onclick = function() {
                if (whoseTurn() == "player") {
                    rsmodel.play(i);
                    bounceView();
                    findWork();
                }
            }
        });
        ["p0", "p1"].forEach(function (name) {
            document.getElementsByName(name).forEach((el) => {
                el.onclick = function() { findWork(); }
            })
        })
        bounceView();
    };

    let findWork = function() {
        let view = rsmodel.viewBoard();
        if (whoseTurn() == "robot") {
            rsmodel.play(pickMove(view.advice));
            bounceView();
        } 
    }

    let bounceView = function() {
        let view = rsmodel.viewBoard();
        let symbolize = function(x) {
            switch (x) {
                case 0: return "&#129415;"; 
                case 1: return "&#129422;"; 
                default: return "&#9898;"; 
            }
        }
        for (var i = 0; i < widgets.length; i++) {
            widgets[i].innerHTML = symbolize(view.cells[i]);
        }
        document.getElementById("p0wants").innerHTML = symbolize(view.wants.p0);
        document.getElementById("p1wants").innerHTML = symbolize(view.wants.p1);
    };

    setUp();
});

function pickMove(advice) {
    let sum = 0.0;
    for (var i = 0; i < advice.length; i++) { sum += advice[i]; }
    let rnd = Math.random() * sum;
    for (var i = 0; i < advice.length; i++) {
        if (advice[i] > rnd) { return i; }
        rnd -= advice[i];
    }
    throw new Error("couldn't find any advice to follow");
}
