import { memory } from "lok-wasm/lok_wasm_bg";
import { Board } from "lok-wasm";

const board = Board.new();
renderBoard(board);

function renderBoard(board) {
    const width = board.width();
    const height = board.height();

    const calculateIndex = (row, column) => {
        return row * width + column;
    };

    const cells = new Uint8Array(memory.buffer, board.cells(), width * height);

    const boardDisplay = document.getElementById("board_display");

    const boardTable = document.createElement("table");
    for (var r = 0; r < height; r++) {
        const row = document.createElement("tr");
        for (var c = 0; c < width; c++) {
            const cell = document.createElement("td");
            cell.className = "normal_cell";
            cell.textContent = String.fromCharCode(cells[calculateIndex(r, c)]);
            row.appendChild(cell);
        }
        boardTable.appendChild(row);
        //console.log("cells[" + i + "] = " + cells[i]);
        //boardDisplay.textContent += String.fromCharCode(cells[i]);
    }

    boardDisplay.replaceChild(boardTable, boardDisplay.firstChild);
}
