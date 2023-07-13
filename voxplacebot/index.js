const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8000/ws/');
const GRID_SIZE = 128;

ws.on('open', async function open() {
    while (true) {
        let action = {
            UpdateVoxel: {
                user: "Client",
                x: getRandomInt(0, GRID_SIZE - 1),
                y: getRandomInt(0, GRID_SIZE - 1),
                z: getRandomInt(0, GRID_SIZE - 1),
                color: getRandomInt(1, 32)
            }
        }
        await new Promise((resolve, reject) => {
            ws.send(JSON.stringify(action), err => {
                if (err) reject(err);
                else resolve();
            });
        });
    }
});

ws.on('message', function incoming(data) {

});

function getRandomInt(min, max) {
    min = Math.ceil(min);
    max = Math.floor(max);
    return Math.floor(Math.random() * (max - min + 1)) + min; // L'inclusivité maximale est minimale
}