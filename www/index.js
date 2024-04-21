import { memory } from "lok-wasm/lok_wasm_bg";
import { Board, BoardCell } from "lok-wasm";

document.getElementById("check_solution").addEventListener("click", onClickCheckSolution);
document.getElementById("generate_form").addEventListener("submit", onGenerateSubmit);

var g_board = null;

function onGenerateSubmit(evt) {
    g_board = Board.new(document.getElementById("puzzle_entry").value);
    renderBoard(g_board);

    const resultDisplay = document.getElementById("result_display");
    resultDisplay.className = null;
    resultDisplay.textContent = "Unsolved";
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
                cell.className = "normal_cell";
                cell.addEventListener("click", onCellClick);
            }

            cell.textContent = boardCell.get_display();
            row.appendChild(cell);
        }
        boardTable.appendChild(row);
    }

    boardDisplay.replaceChild(boardTable, boardDisplay.firstChild);
}

document.getElementById("generate_form").requestSubmit();
