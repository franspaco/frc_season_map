red2blue = [
    "#ff0000",
    "#f8003b",
    "#f10053",
    "#ea0065",
    "#e30075",
    "#db0083",
    "#d4008f",
    "#cb009b",
    "#c300a5",
    "#ba00b0",
    "#b100b9",
    "#a700c2",
    "#9d00cb",
    "#9100d3",
    "#8500db",
    "#7800e3",
    "#6900ea",
    "#5700f1",
    "#4100f8",
    "#1e00ff",
];

function getColor(min, max, value) {
    let array = red2blue;
    let rel = (value - min) / (max - min);
    if (rel < 0) {
        return array[0];
    } else if (rel > 1) {
        return array[array.length - 1];
    } else {
        return array[Math.round(rel * array.length)];
    }
}
