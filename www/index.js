import { peek } from "./rsmodel.js";

import("./rsmodel.js").then(rsmodel => {
    let container = document.getElementById("container");
    container.innerHTML = `
        <div id="title" class="state">
            <h1>&#129415&#10084;&#65039&#129422;</h1> <!-- "Bat Likes Kobold" -->
            <p>by Nyeogmi and Pyrex</p>
            <ul>
                <li><a class="button goto" goto="notice1">Play (with tutorial)</a></li>
                <li><a class="button goto" goto="game">Skip intro</a></li>
            </ul>
        </div>
        <div id="notice1" class="state">
            <p>You're a BAT!</p>
            <p>That reptile over there is your best friend.</p>
            <p>... And it wants to beat you at Tic-Tac-Toe. You think.</p>
            <a class="button goto" goto="notice2">... Yeah?</a>
        </div>
        <div id="notice2" class="state">
            <p>Actually, the two of you have spent so much time letting each other win that it's no longer clear what anyone wants.</p>
            <p>Each round you're told one of three things, at random:</p>
            <ul>
                <li>&#129415; You want yourself to get three in a row.</li>
                <li>&#129422; You want Pyrex to three in a row.</li>
                <li>&#9898; You're going for a draw.</li>
            </ul>
            <p>Pyrex has also been given a random outcome from that list.</p>
            <p>Each player's wincon is secret.</p>
            <a class="button goto" goto="notice1">Back</a>
            <a class="button goto" goto="notice3">Right.</a>
        </div>
        <div id="notice3" class="state">
            <p>If you get what you wanted, you get a point.</p>
            <p>If Pyrex gets what it wanted, Pyrex gets a point.</p>
            <p>If the two of you wanted the same thing, then you _both_ get a point.</p>
            <p>If neither of you wanted the outcome that occurred, neither of you gets a point.</p>
            <p>Then the next round happens and you alternate who goes first.</p>
            <a class="button goto" goto="notice2">Back</a>
            <a class="button goto" goto="game">OK! I'll play</a>
        </div>
        <div id="game" class="state">
            <table id="score">
                <tr>
                    <td>&#129415; <span id="batScore">0</span></td>
                    <td>&#129422; <span id="kobScore">0</span></td>
                </tr>
            </table>
            <table id="gameArea">
                <tr><td id="c0"/><td id="c1"/><td id="c2"/></tr>
                <tr><td id="c3"/><td id="c4"/><td id="c5"/></tr>
                <tr><td id="c6"/><td id="c7"/><td id="c8"/></tr>
            </table>
            <table id="wants">
                <tr>
                    <td id="batVerdict">&#129415; wants <span id="batWants"></span></td>
                    <td id="kobVerdict">&#129422; wants <span id="kobWants"></span>
                    </td>
                </tr>
            </table>
            <div id="gutter">
                <a class="button" id="hint">HINT???</a>
            </div>
            <div id="gutter2">
                <div id="nextGameRow" class="inactive"><a class="button" id="nextGameButton">NEXT GAME</a></div>
                <a class="button" id="peek">Peek (CHEATING)</a>
            </div>
        </div>
    `
    // !!! UI: Store references to all pats
    let gameStates = [];
    for (var el of document.getElementsByClassName("state")) { gameStates.push(el); }
    let gotoButtons = [];
    for (var el of document.getElementsByClassName("goto")) { gotoButtons.push(el); }
    let cells = [];
    ["c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8"].forEach(function(el, i) { cells.push(document.getElementById(el)) });
    let peekButton = document.getElementById("peek");
    let hintButton = document.getElementById("hint");
    let nextGameButton = document.getElementById("nextGameButton");
    let nextGameRow = document.getElementById("nextGameRow");

    let playerWidgets = {
        bat: {
            score: document.getElementById("batScore"), 
            wants: document.getElementById("batWants"), 
            verdict: document.getElementById("batVerdict"),
        },
        kob: {
            score: document.getElementById("kobScore"),
            wants: document.getElementById("kobWants"), 
            verdict: document.getElementById("kobVerdict"),
        },
    };

    // !!! UI: Initialize state info
    let activeState = "title";
    let players = {
        bat: {
            identity: 1,
            score: 0,
        },
        kob: {
            identity: 0,
            score: 0,
        }
    }

    // !!! TUTORIAL: set up event handlers
    gotoButtons.forEach((btn) => {
        btn.onclick = () => {
            console.log(btn.getAttribute("goto"));
            activeState = btn.getAttribute("goto");
            findWork();
            bounceView();
        }
    })

    // !!! GAME: set up event handlers. controller code etc
    cells.forEach((widg, i) => {
        widg.onclick = function() {
            if (whoseTurn() == "player") {
                rsmodel.play(i, handleProgress);
                bounceView();
                findWork();
            }
        }
    });

    peekButton.onclick = function() { 
        if (peekButton.classList.contains("disabled")) { return; }
        rsmodel.peek(); bounceView(); 
    }
    hintButton.onclick = function() { 
        if (hintButton.classList.contains("disabled")) { return; }
        rsmodel.hint(); bounceView(); 
    }
    nextGameButton.onclick = function() {
        players.bat.identity = 1 - players.bat.identity;
        players.kob.identity = 1 - players.kob.identity;
        rsmodel.start();
        bounceView();
        findWork();
    }

    let whoseTurn = function() {
        let view = rsmodel.viewBoard();
        if (view.outcome != null) { return "nobody"; }
        if (view.playerTurn == players.bat.identity) { return "player"; }
        return "robot"; 
    }

    let findWork = function() {
        let view = rsmodel.viewBoard();
        if (whoseTurn() == "robot") {
            rsmodel.play(pickMove(view.advice), handleProgress);
            bounceView();
        } 
    }

    let handleProgress = function(view) {
        if (view.outcome != null) {
            players.bat.score += view.outcome.util[players.bat.identity];
            players.kob.score += view.outcome.util[players.kob.identity];
        }
    }

    // !!! GAME: View update code 
    let bounceView = function() {
        gameStates.forEach(i => {
            console.log(i)
            if (i.id == activeState) {
                i.classList.remove("inactive")
            } else {
                i.classList.add("inactive")
            }
        });

        // dump board
        let view = rsmodel.viewBoard();
        let symbolize = function(x, show) {
            if (!show) {
                return "&#10084;&#65039";
            }
            if (x == players.bat.identity) { return "&#129415;"; }
            if (x == players.kob.identity) { return "&#129422;"; }
            return "&#9898;"; 
        }
        for (var i = 0; i < cells.length; i++) {
            cells[i].innerHTML = symbolize(view.cells[i], true);
        }
        if (view.hinted) {
            let advice = view.advice;
            let max = Math.max(...advice);
            if (max == 0) {
                for (var i = 0; i < cells.length; i++) { cells[i].style.backgroundColor = ""; }
            } else {
                for (var i = 0; i < cells.length; i++) {
                    cells[i].style.backgroundColor = "hsl(" + advice[i]/max * 120 + ",100%, " + (40 + advice[i]/max * 10) + "%," + advice[i]/max * 100 + "%)"
                }
            }
        } else {
            for (var i = 0; i < cells.length; i++) { cells[i].style.backgroundColor = ""; }
        }
        playerWidgets.bat.wants.innerHTML = symbolize(view.wants[players.bat.identity], true);
        playerWidgets.kob.wants.innerHTML = symbolize(view.wants[players.kob.identity], view.peeked || view.outcome != null);

        if (view.peeked || view.outcome != null) {
            peekButton.classList.add("disabled")
        } else {
            peekButton.classList.remove("disabled");
        }
        peekButton.innerHTML = !view.peeked ? "Peek<br />(CHEATING)" : "CHEATER";

        if (hintButton.disabled = view.hinted || view.outcome != null || whoseTurn() != "player") {
            hintButton.classList.add("disabled")
        } else {
            hintButton.classList.remove("disabled");
        }
        hintButton.innerHTML = !view.hinted ? "HINT???" : "OK LOOK AT THE BOARD";

        playerWidgets.bat.score.innerHTML = "" + players.bat.score;
        playerWidgets.kob.score.innerHTML = "" + players.kob.score;

        if (view.outcome != null && view.outcome.util[players.bat.identity] > 0) {
            playerWidgets.bat.verdict.classList.add("verdict-won");
        } else {
            playerWidgets.bat.verdict.classList.remove("verdict-won");
        }

        if (view.outcome != null && view.outcome.util[players.kob.identity] > 0) {
            playerWidgets.kob.verdict.classList.add("verdict-won");
        } else {
            playerWidgets.kob.verdict.classList.remove("verdict-won");
        }
        
        if (view.outcome == null) {
            nextGameRow.classList.add("inactive");
        } else {
            nextGameRow.classList.remove("inactive");
        }
    };

    rsmodel.start();
    bounceView();
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
