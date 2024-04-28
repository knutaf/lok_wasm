import { Board, BoardCell } from "lok-wasm";

window.addEventListener("hashchange", onHashChange);
window.addEventListener("keydown", onKeyDown);
document.getElementById("check_solution").addEventListener("click", onClickCheckSolution);
document.getElementById("render_form").addEventListener("submit", onRenderSubmit);
document.getElementById("undo").addEventListener("click", onClickUndo);

{
    const modeElements = document.getElementsByName("mode");
    for (var i = 0; i < modeElements.length; i++)
    {
        modeElements[i].addEventListener("change", onModeChange);
    }
}

var g_lastModeEditState = false;

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
            encodedPuzzle = encodeURIComponent("LO-_K-\nL_O_K_\nTLAK--");
        }

        document.getElementById("puzzle_entry").value = decodeURIComponent(encodedPuzzle);
        setPuzzle();
    }
}

function onKeyDown(evt) {
    switch (evt.key) {
        // ALT-z to undo.
        case "z": {
            if (evt.altKey) {
                onClickUndo();
            }
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
                        onModeChange();
                        break;
                    }
                }
            }
            break;
        }
    }
}

function onModeChange(evt) {
    const nowInModeEdit = document.getElementById("modeEdit").checked;
    if (nowInModeEdit != g_lastModeEditState) {
        g_lastModeEditState = nowInModeEdit;
        renderBoard();
    }
}

function setPuzzle() {
    const puzzle = document.getElementById("puzzle_entry").value;
    try {
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
    catch (ex) {
        alert("Error rendering puzzle: " + ex);
    }
}

function onRenderSubmit(evt) {
    setPuzzle();
    return false;
}

function getMode() {
    if (document.getElementById("modeBlacken").checked) {
        return "blacken";
    } else if (document.getElementById("modeMarkPath").checked) {
        return "markPath";
    } else if (document.getElementById("modeEdit").checked) {
        return "modeEdit";
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

function onLetterFocus(evt) {
    const target = evt.currentTarget;
    window.getSelection().selectAllChildren(target);
}

function onLetterInput(evt) {
    const target = evt.currentTarget;
    const cell = target.parentElement;

    const letterText = target.textContent;
    if (letterText.length > 0) {
        g_board.change_letter(cell.boardRow, cell.boardCol, letterText.charAt(0));
        renderBoard();
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
    const currentMode = getMode();
    const isInEditMode = (currentMode == "modeEdit");

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

                const letterDisplay = document.createElement("span");
                letterDisplay.textContent = boardCell.get_display();

                if (isInEditMode) {
                    letterDisplay.classList.add("editable_letter_display");
                    letterDisplay.contentEditable = "plaintext-only";
                    letterDisplay.addEventListener("focus", onLetterFocus);
                    letterDisplay.addEventListener("input", onLetterInput);
                } else {
                    letterDisplay.contentEditable = "false";
                }

                const markCountDisplay = document.createElement("sup");

                const markCount = boardCell.get_mark_count();
                if (markCount > 1) {
                    markCountDisplay.textContent = "" + markCount;
                } else {
                    markCountDisplay.textContent = " ";
                }

                cell.addEventListener("click", onCellClick);

                cell.appendChild(letterDisplay);
                cell.appendChild(markCountDisplay);
            }

            if (boardCell.is_blackened()) {
                cell.classList.add("blackened");
            }

            if (boardCell.is_marked_for_path()) {
                cell.classList.add("pathmarked");
            }
            row.appendChild(cell);
        }
        boardTable.appendChild(row);
    }

    boardDisplay.replaceChild(boardTable, boardDisplay.firstChild);
}

onHashChange();
