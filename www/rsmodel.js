import * as wasm from "tictac2player";

wasm.init();

let board = null;
let temporary = null;

let start = function() {
    if (board != null) { board.free() }
    board = wasm.Board.js_start_random()
    temporary = {
        hinted: false,
        peeked: false,
    }
}

let viewBoard = function() {
    let view = board.js_view();
    let outcome = view.outcome;
    let advice = [];
    let cells = [];
    for (var i = 0; i < 9; i++) {
        advice.push(view.get_advice(i));

        let value = view.get_cell(i);
        cells.push(
            value == 0 ? 0 :
            value == 1 ? 1 :
            null
        )
    }
    let view_data = {
        playerTurn: view.player_turn,
        outcome: outcome == 255 ? null : {
            winner: 
                outcome == 0 ? 0 :
                outcome == 1 ? 1 :
                outcome == 2 ? null :
                "wtf",
            util: [view.util_p0, view.util_p1]
        },
        wants: [view.wants_p0, view.wants_p1],
        advice: advice,
        cells: cells,
        hinted: temporary.hinted,
        peeked: temporary.peeked,
    };
    view.free()
    return view_data;
}

let hint = function() {
    temporary.hinted = true;
}

let peek = function() {
    temporary.peeked = true;
}

let play = function(cell, progressedCb) {
    let old_view = board.js_view();
    board.js_play(cell)
    let new_view = board.js_view();
    let progressed = old_view.player_turn != new_view.player_turn;
    if (progressed) {
        temporary.hinted = false
        progressedCb(viewBoard())
    }
}

export {
    start,
    viewBoard,
    hint,
    peek,
    play,
};