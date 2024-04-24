import { memory } from "lok-wasm/lok_wasm_bg";
import { Board, BoardCell } from "lok-wasm";

window.addEventListener("hashchange", onHashChange);
window.addEventListener("keydown", onKeyDown);
document.getElementById("check_solution").addEventListener("click", onClickCheckSolution);
document.getElementById("generate_form").addEventListener("submit", onGenerateSubmit);
document.getElementById("undo").addEventListener("click", onClickUndo);

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

function onKeyDown(evt) {
    switch (evt.key) {
        // Z to undo.
        case "z": {
            onClickUndo();
            break;
        }

        // CTRL-Enter to set puzzle, when the puzzle text entry is in focus.
        case "Enter": {
            if (evt.ctrlKey) {
                if (document.activeElement == document.getElementById("puzzle_entry")) {
                    setPuzzle();
                }
            }
            break;
        }

        // CTRL-m to switch modes.
        case "m": {
            if (evt.ctrlKey) {
                const modeElements = document.getElementsByName("mode");
                for (var i = 0; i < modeElements.length; i++)
                {
                    if (modeElements[i].checked) {
                        modeElements[(i + 1) % modeElements.length].checked = true;
                        break;
                    }
                }
            }
            break;
        }
    }
}

function setPuzzle() {
    const puzzle = document.getElementById("puzzle_entry").value;
    g_board = Board.new(puzzle);
    renderBoard();

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

function getMode() {
    if (document.getElementById("modeBlacken").checked) {
        return "blacken";
    } else if (document.getElementById("modeMarkPath").checked) {
        return "markPath";
    }
}

function onCellClick(evt) {
    const cell = evt.currentTarget;
    switch (getMode()) {
        case "blacken": {
            g_board.blacken(cell.boardRow, cell.boardCol);
            renderBoard();
            break;
        }
        case "markPath": {
            g_board.mark_path(cell.boardRow, cell.boardCol);
            renderBoard();
            break;
        }
    }
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

function onClickUndo(evt) {
    g_board.undo();
    renderBoard();
}

function renderBoard() {
    const width = g_board.width();
    const height = g_board.height();

    const boardDisplay = document.getElementById("board_display");

    const boardTable = document.createElement("table");
    for (var r = 0; r < height; r++) {
        const row = document.createElement("tr");
        for (var c = 0; c < width; c++) {
            const boardCell = g_board.get(r, c);

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

            if (boardCell.is_marked_for_path()) {
                cell.classList.add("pathmarked");
            }

            const letter = document.createElement("span");
            letter.textContent = boardCell.get_display();

            const markCountDisplay = document.createElement("sup");

            const markCount = boardCell.get_mark_count();
            if (markCount > 1) {
                markCountDisplay.textContent = "" + markCount;
            } else {
                markCountDisplay.textContent = " ";
            }

            cell.appendChild(letter);
            cell.appendChild(markCountDisplay);
            row.appendChild(cell);
        }
        boardTable.appendChild(row);
    }

    boardDisplay.replaceChild(boardTable, boardDisplay.firstChild);
}

onHashChange();
