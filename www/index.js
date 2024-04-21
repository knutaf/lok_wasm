import { memory } from "lok-wasm/lok_wasm_bg";
import { Board, BoardCell } from "lok-wasm";

window.addEventListener("hashchange", onHashChange);
document.getElementById("check_solution").addEventListener("click", onClickCheckSolution);
document.getElementById("generate_form").addEventListener("submit", onGenerateSubmit);

var g_anchor = null;
var g_board = null;

// If the hash/anchor of the URL has changed, load the newly specified puzzle
function onHashChange() {
    const newAnchor = window.location.hash;
    if (newAnchor != g_anchor) {
        g_anchor = newAnchor;
        const anchorStart = g_anchor.indexOf("#");

        var encodedPuzzle = "";
        if (anchorStart != -1) {
            encodedPuzzle = g_anchor.substring(anchorStart + 1);
        }

        if (encodedPuzzle == "") {
            encodedPuzzle = encodeURIComponent("LO_ K \nL_O K \nTLAK__");
        }

        document.getElementById("puzzle_entry").value = decodeURIComponent(encodedPuzzle);
        setPuzzle();
    }
}

function setPuzzle() {
    const puzzle = document.getElementById("puzzle_entry").value;
    g_board = Board.new(puzzle);
    renderBoard(g_board);

    const resultDisplay = document.getElementById("result_display");
    resultDisplay.className = null;
    resultDisplay.textContent = "Unsolved";

    var newHash = "#" + encodeURIComponent(puzzle);
    if (window.location.hash != newHash) {
        console.log("setting hash to " + newHash);
        window.location.hash = newHash;
    }
}

function onGenerateSubmit(evt) {
    setPuzzle();
    return false;
}

function onCellClick(evt) {
    const cell = evt.target;
    g_board.blacken(cell.boardRow, cell.boardCol);
    cell.classList.add("blackened");
}

function onClickCheckSolution(evt) {
    const result = g_board.commit_and_check_solution();
    const resultDisplay = document.getElementById("result_display");
    if (result == null) {
        resultDisplay.className = "result_success";
        resultDisplay.textContent = "YAY";
    } else {
        resultDisplay.className = "result_fail";
        resultDisplay.textContent = "NAY";
    }
}

function renderBoard(board) {
    const width = board.width();
    const height = board.height();

    const boardDisplay = document.getElementById("board_display");

    const boardTable = document.createElement("table");
    for (var r = 0; r < height; r++) {
        const row = document.createElement("tr");
        for (var c = 0; c < width; c++) {
            const boardCell = board.get(r, c);

            const cell = document.createElement("td");
            cell.boardRow = r;
            cell.boardCol = c;

            if (boardCell.is_interactive()) {
                cell.classList.add("normal_cell");
                cell.addEventListener("click", onCellClick);
            }

            if (boardCell.is_blackened()) {
                cell.classList.add("blackened");
            }

            cell.textContent = boardCell.get_display();
            row.appendChild(cell);
        }
        boardTable.appendChild(row);
    }

    boardDisplay.replaceChild(boardTable, boardDisplay.firstChild);
}

onHashChange();
