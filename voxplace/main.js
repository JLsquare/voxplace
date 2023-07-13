import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import {GLTFLoader} from 'three/addons/loaders/GLTFLoader.js';
import Stats from 'stats.js'

const size = 128;
const chunkSize = 16;
const moveSpeed = 0.5;
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 2000);
const renderer = new THREE.WebGLRenderer({ antialias: true });
const controls = new OrbitControls(camera, renderer.domElement);
const loader = new GLTFLoader();
const stats = new Stats();

controls.maxDistance = 512;
controls.minDistance = 2;

const boxGeometry = new THREE.BoxGeometry(1, 1, 1);
const invertedBoxGeometry = new THREE.BoxGeometry(size, size, size);
invertedBoxGeometry.applyMatrix4(new THREE.Matrix4().makeScale(-1, -1, -1));
const planeGeometry = new THREE.PlaneGeometry(size, size);
planeGeometry.applyMatrix4(new THREE.Matrix4().makeRotationX(-Math.PI / 2));

let voxels = [];
let palette = [];
let selectMesh = null;
let previewLeftClickMesh = null;
let previewRightClickMesh = null;
let chunks = [];

const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();
let mouseDown = new THREE.Vector2();
let mouseUp = new THREE.Vector2();
let clickCount = 0;
let clickTime = 0;
let currentTime = 0;
let timeLimit = 500;
let oldClick = new THREE.Vector2();
let selectedColor = null;
let needsUpdate = [];
let xUpdate = 0;
let yUpdate = 0;
let zUpdate = 0;

let socket = new WebSocket(`ws://${window.location.hostname}:8000/api/place/temp/ws/`);

initStats();
initScene();
initPalette();
await initVoxelData();
initChunks();
initSocket();
animate();
addEventListeners();

function initSocket() {
    socket.onopen = () => {
        console.log('[open] Connection established');
    };

    socket.onerror = (error) => {
        console.log(`[error] ${error.message}`);
    }

    socket.onmessage = (event) => {
        let data = JSON.parse(event.data);
        if (data.action === 'update') {
            updateVoxel(data.x, data.y, data.z, data.color);
        }
    }

    socket.onclose = (event) => {
        if (event.wasClean) {
            console.log(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
            console.log('[close] Connection died');
        }
    }
}

function initStats() {
    stats.showPanel(0);
    document.body.appendChild(stats.dom);
}

function initScene() {
    const borderMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.1 });
    const borderMesh = new THREE.Mesh(invertedBoxGeometry, borderMaterial);
    borderMesh.position.set(-0.5, -0.5, -0.5);
    //disable raycast
    borderMesh.raycast = () => [];
    scene.add(borderMesh);

    const floorMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.1, depthTest: false });
    const floorMesh = new THREE.Mesh(planeGeometry, floorMaterial);
    floorMesh.position.set(-0.5, -size / 2, -0.5);
    floorMesh.renderOrder = 1;
    scene.add(floorMesh);

    const edgesGeometry = new THREE.EdgesGeometry(invertedBoxGeometry);
    const edgesMaterial = new THREE.LineBasicMaterial({ color: 0xffffff });
    const edgesMesh = new THREE.LineSegments(edgesGeometry, edgesMaterial);
    edgesMesh.position.set(-0.5, -0.5, -0.5);
    scene.add(edgesMesh);

    edgesMesh.raycast = () => [];

    loader.load('selected.glb', (gltf) => {
        const selectMaterial = new THREE.MeshBasicMaterial({ color: 0xfffffff, depthTest: false });
        const selectGeometry = gltf.scene.children[0].geometry;
        const previewLeftClickMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.5, depthTest: false });
        const previewRightClickMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.25, depthTest: false });
        selectMesh = new THREE.Mesh(selectGeometry, selectMaterial);
        selectMesh.scale.set(0.05, 0.05, 0.05);
        selectMesh.renderOrder = 2;
        scene.add(selectMesh);
        selectMesh.visible = false;
        previewLeftClickMesh = new THREE.Mesh(selectGeometry, previewLeftClickMaterial);
        previewLeftClickMesh.scale.set(0.05, 0.05, 0.05);
        previewLeftClickMesh.renderOrder = 3;
        previewRightClickMesh = new THREE.Mesh(selectGeometry, previewRightClickMaterial);
        previewRightClickMesh.scale.set(0.05, 0.05, 0.05);
        previewRightClickMesh.renderOrder = 3;
        scene.add(previewLeftClickMesh);
        scene.add(previewRightClickMesh);
    });

    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.domElement.id = 'canvas';
    document.body.appendChild(renderer.domElement);

    camera.position.set(size / 2, size / 2, size / 2);

    const skyboxGeometry = new THREE.IcosahedronGeometry(1000, 4);
    const skyboxMaterial = new THREE.MeshBasicMaterial( { map: createGradientTexture([[0.75,0.6,0.4,0.25], ['#1B1D1E','#3D4143','#72797D', '#b0babf']], 1024, 1024), side:THREE.BackSide, depthWrite: false }  );
    const back = new THREE.Mesh( skyboxGeometry, skyboxMaterial);
    scene.add( back );
}

function initPalette() {
    let colorControls = document.getElementById('color-controls');
    palette = [
        '#6d001a', '#be0039', '#ff4500', '#ffa800', '#ffd635', '#fff8b8',
        '#00a368', '#00cc78', '#7eed56', '#00756f', '#009eaa', '#00ccc0',
        '#2450a4', '#3690ea', '#51e9f4', '#493ac1', '#6a5cff', '#94b3ff',
        '#811e9f', '#b44ac0', '#e4abff', '#de107f', '#ff3881', '#ff99aa',
        '#6d482f', '#9c6926', '#ffb470', '#000000', '#515252', '#898d90',
        '#d4d7d9', '#ffffff', 'empty'
    ];

    palette.forEach((color, index) => {
        let input = document.createElement('input');
        input.type = "radio";
        input.id = "color" + index;
        input.name = "color";
        input.value = color;
        input.classList.add("color-box");
        input.addEventListener('change', changeColor);

        let label = document.createElement('label');
        label.htmlFor = "color" + index;
        if (color === 'empty') {
            label.style.backgroundImage = "url('/remove.png')";
            label.style.backgroundSize = 'cover';
        } else {
            label.style.backgroundColor = color;
        }
        label.classList.add("color-label");

        colorControls.appendChild(input);
        colorControls.appendChild(label);
    });

    palette = palette.map(color => color === 'empty' ? 'empty' : new THREE.Color(color));
}

async function initVoxelData() {
    await fetch(`http://${window.location.hostname}:8000/api/place/temp/all`)
        .then(response => response.arrayBuffer())
        .then(data => {
            const bytes = new Uint8Array(data);
            for(let x = 0; x < size; x++) {
                voxels[x] = [];
                for(let y = 0; y < size; y++) {
                    voxels[x][y] = [];
                    for(let z = 0; z < size; z++) {
                        const value = bytes[x * size * size + y * size + z];
                        if(value > 0) {
                            voxels[x][y][z] = palette[value - 1];
                        } else {
                            voxels[x][y][z] = null;
                        }
                    }
                }
            }
        });
}

function initChunks() {
    for(let x = 0; x < size / chunkSize; x++) {
        chunks[x] = [];
        needsUpdate[x] = [];
        for(let y = 0; y < size / chunkSize; y++) {
            chunks[x][y] = [];
            needsUpdate[x][y] = [];
            for(let z = 0; z < size / chunkSize; z++) {
                needsUpdate[x][y][z] = false;
                let chunkMesh = generateChunk(x, y, z);
                if (chunkMesh) {
                    chunks[x][y][z] = chunkMesh;
                    scene.add(chunkMesh);
                }
            }
        }
    }
}

function generateChunk(x, y, z) {
    let chunkGeometry = new THREE.BufferGeometry();
    let chunkMaterial;
    let chunkMesh;
    let chunkVertices = [];
    let chunkColors = [];
    let indexOffset = 0;
    let chunkIndices = [];

    x *= chunkSize;
    y *= chunkSize;
    z *= chunkSize;

    for (let dx = 0; dx < chunkSize; dx++) {
        for (let dy = 0; dy < chunkSize; dy++) {
            for (let dz = 0; dz < chunkSize; dz++) {
                const voxel = voxels[x + dx][y + dy][z + dz];
                if (voxel !== null) {
                    let leftEmpty = x + dx - 1 < 0 || voxels[x + dx - 1][y + dy][z + dz] === null;
                    let rightEmpty = x + dx + 1 >= size || voxels[x + dx + 1][y + dy][z + dz] === null;
                    let bottomEmpty = y + dy - 1 < 0 || voxels[x + dx][y + dy - 1][z + dz] === null;
                    let topEmpty = y + dy + 1 >= size || voxels[x + dx][y + dy + 1][z + dz] === null;
                    let frontEmpty = z + dz - 1 < 0 || voxels[x + dx][y + dy][z + dz - 1] === null;
                    let backEmpty = z + dz + 1 >= size || voxels[x + dx][y + dy][z + dz + 1] === null;

                    if(leftEmpty){
                        chunkVertices.push(
                            -0.5 + dx, -0.5 + dy,  0.5 + dz,
                            -0.5 + dx,  0.5 + dy,  0.5 + dz,
                            -0.5 + dx,  0.5 + dy, -0.5 + dz,
                            -0.5 + dx, -0.5 + dy, -0.5 + dz
                        );
                        chunkIndices.push(indexOffset, indexOffset + 1, indexOffset + 2, indexOffset, indexOffset + 2, indexOffset + 3);
                        indexOffset += 4;
                    }

                    if(rightEmpty) {
                        chunkVertices.push(
                            0.5 + dx, -0.5 + dy, -0.5 + dz,
                            0.5 + dx,  0.5 + dy, -0.5 + dz,
                            0.5 + dx,  0.5 + dy,  0.5 + dz,
                            0.5 + dx, -0.5 + dy,  0.5 + dz
                        );
                        chunkIndices.push(indexOffset, indexOffset + 1, indexOffset + 2, indexOffset, indexOffset + 2, indexOffset + 3);
                        indexOffset += 4;
                    }

                    if(bottomEmpty) {
                        chunkVertices.push(
                            -0.5 + dx, -0.5 + dy, -0.5 + dz,
                            0.5 + dx, -0.5 + dy, -0.5 + dz,
                            0.5 + dx, -0.5 + dy,  0.5 + dz,
                            -0.5 + dx, -0.5 + dy,  0.5 + dz
                        );
                        chunkIndices.push(indexOffset, indexOffset + 1, indexOffset + 2, indexOffset, indexOffset + 2, indexOffset + 3);
                        indexOffset += 4;
                    }

                    if(topEmpty) {
                        chunkVertices.push(
                            -0.5 + dx,  0.5 + dy,  0.5 + dz,
                            0.5 + dx,  0.5 + dy,  0.5 + dz,
                            0.5 + dx,  0.5 + dy, -0.5 + dz,
                            -0.5 + dx,  0.5 + dy, -0.5 + dz
                        );
                        chunkIndices.push(indexOffset, indexOffset + 1, indexOffset + 2, indexOffset, indexOffset + 2, indexOffset + 3);
                        indexOffset += 4;
                    }

                    if(frontEmpty) {
                        chunkVertices.push(
                            0.5 + dx, -0.5 + dy, -0.5 + dz,
                            -0.5 + dx, -0.5 + dy, -0.5 + dz,
                            -0.5 + dx,  0.5 + dy, -0.5 + dz,
                            0.5 + dx,  0.5 + dy, -0.5 + dz
                        );
                        chunkIndices.push(indexOffset, indexOffset + 1, indexOffset + 2, indexOffset, indexOffset + 2, indexOffset + 3);
                        indexOffset += 4;
                    }

                    if(backEmpty) {
                        chunkVertices.push(
                            0.5 + dx, -0.5 + dy,  0.5 + dz,
                            0.5 + dx,  0.5 + dy,  0.5 + dz,
                            -0.5 + dx,  0.5 + dy,  0.5 + dz,
                            -0.5 + dx, -0.5 + dy,  0.5 + dz
                        );
                        chunkIndices.push(indexOffset, indexOffset + 1, indexOffset + 2, indexOffset, indexOffset + 2, indexOffset + 3);
                        indexOffset += 4;
                    }

                    for(let i = 0; i < 6; i++) {
                        if([leftEmpty, rightEmpty, bottomEmpty, topEmpty, frontEmpty, backEmpty][i]) {
                            chunkColors.push(voxel.r, voxel.g, voxel.b);
                            chunkColors.push(voxel.r, voxel.g, voxel.b);
                            chunkColors.push(voxel.r, voxel.g, voxel.b);
                            chunkColors.push(voxel.r, voxel.g, voxel.b);
                        }
                    }
                }
            }
        }
    }

    chunkGeometry.setAttribute('position', new THREE.Float32BufferAttribute(chunkVertices, 3));
    chunkGeometry.setIndex(chunkIndices);
    chunkGeometry.setAttribute('color', new THREE.Float32BufferAttribute(chunkColors, 3));

    chunkMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, vertexColors: true });
    chunkMesh = new THREE.Mesh(chunkGeometry, chunkMaterial);
    chunkMesh.position.set(x - size / 2, y - size / 2, z - size / 2);

    return chunkMesh;
}

function animate() {
    stats.begin();
    requestAnimationFrame(animate);
    const frustum = new THREE.Frustum();
    frustum.setFromProjectionMatrix(new THREE.Matrix4().multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse));
    for(let x = 0; x < size / chunkSize; x++) {
        for(let y = 0; y < size / chunkSize; y++) {
            for(let z = 0; z < size / chunkSize; z++) {
                if(chunks[x][y][z]) {
                    if(frustum.intersectsObject(chunks[x][y][z])){
                        chunks[x][y][z].visible = true;
                    } else {
                        chunks[x][y][z].visible = false;
                    }
                }
            }
        }
    }
    updateWhenNeeded();
    controls.update();
    renderer.render(scene, camera);
    stats.end();
}

function addEventListeners() {
    document.addEventListener('keydown', handleKeyDown, false);
    document.addEventListener('mousemove', handleMouseMove, false);
    document.addEventListener('mousedown', handleMouseDown, false);
    document.addEventListener('mouseup', handleMouseUp, false);
    document.getElementById('edit-voxel-btn').addEventListener('click', editVoxel, false);
}

function handleKeyDown(event) {
    const oldPosition = camera.position.clone();
    switch (event.key) {
        case "w":
        case "z":
            camera.translateZ(-moveSpeed);
            break;
        case "a":
        case "q":
            camera.translateX(-moveSpeed);
            break;
        case "s":
            camera.translateZ(moveSpeed);
            break;
        case "d":
            camera.translateX(moveSpeed);
            break;
    }
    const newPosition = camera.position.clone();
    controls.target = controls.target.sub(oldPosition).add(newPosition);
}

function handleMouseMove(event) {
    if(event.target.id !== 'canvas') {
        return;
    }
    mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
    mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;

    mouseUp.x = event.clientX;
    mouseUp.y = event.clientY;

    raycaster.setFromCamera(mouse, camera);
    const intersects = raycaster.intersectObjects(scene.children, true);

    if (intersects.length > 0) {
        if(intersects[0].object !== selectMesh && intersects[0].object !== previewLeftClickMesh && intersects[0].object !== previewRightClickMesh) {
            const selectedVoxel = intersects[0];
            let leftClickPosition;
            let rightClickPosition;
            leftClickPosition = new THREE.Vector3(
                Math.round(selectedVoxel.point.x - selectedVoxel.face.normal.x / 2),
                Math.round(selectedVoxel.point.y - selectedVoxel.face.normal.y / 2),
                Math.round(selectedVoxel.point.z - selectedVoxel.face.normal.z / 2)
            );
            rightClickPosition = new THREE.Vector3(
                Math.round(selectedVoxel.point.x + selectedVoxel.face.normal.x / 2),
                Math.round(selectedVoxel.point.y + selectedVoxel.face.normal.y / 2),
                Math.round(selectedVoxel.point.z + selectedVoxel.face.normal.z / 2)
            );
            if(leftClickPosition.y <= -size / 2) {
                rightClickPosition.y = -size / 2;
            }

            if (leftClickPosition.x < -size / 2) { leftClickPosition.x = -size / 2; }
            if (leftClickPosition.x > size / 2 - 1) { leftClickPosition.x = size / 2 - 1; }
            if (leftClickPosition.y < -size / 2) { leftClickPosition.y = -size / 2; }
            if (leftClickPosition.y > size / 2 - 1) { leftClickPosition.y = size / 2 - 1; }
            if (leftClickPosition.z < -size / 2) { leftClickPosition.z = -size / 2; }
            if (leftClickPosition.z > size / 2 - 1) { leftClickPosition.z = size / 2 - 1; }

            if (rightClickPosition.x < -size / 2) { rightClickPosition.x = -size / 2; }
            if (rightClickPosition.x > size / 2 - 1) { rightClickPosition.x = size / 2 - 1; }
            if (rightClickPosition.y < -size / 2) { rightClickPosition.y = -size / 2; }
            if (rightClickPosition.y > size / 2 - 1) { rightClickPosition.y = size / 2 - 1; }
            if (rightClickPosition.z < -size / 2) { rightClickPosition.z = -size / 2; }
            if (rightClickPosition.z > size / 2 - 1) { rightClickPosition.z = size / 2 - 1; }

            leftClickPosition.add(new THREE.Vector3(0.5, 0.5, 0));
            previewLeftClickMesh.position.copy(leftClickPosition);

            rightClickPosition.add(new THREE.Vector3(0.5, 0.5, 0));
            previewRightClickMesh.position.copy(rightClickPosition);
        }
    }
}

function handleMouseDown(event) {
    if(event.target.id !== 'canvas') {
        return;
    }
    mouseDown.x = event.clientX;
    mouseDown.y = event.clientY;
}

function handleMouseUp(event) {
    if(event.target.id !== 'canvas') {
        return;
    }

    currentTime = new Date().getTime();

    if (clickCount > 0 && currentTime - clickTime > timeLimit) {
        clickCount = 0;
    }

    if (mouseDown.distanceTo(mouseUp) < 0.01) {
        if (selectMesh.visible === false) {
            selectMesh.visible = true;
        }
        if(event.button === 0) {
            selectMesh.position.copy(previewLeftClickMesh.position);
        } else if(event.button === 2) {
            selectMesh.position.copy(previewRightClickMesh.position);
        }

        let x = Math.round(selectMesh.position.x + size / 2 - 0.5);
        let y = Math.round(selectMesh.position.y + size / 2 - 0.5);
        let z = Math.round(selectMesh.position.z + size / 2 - 0.5);
        let infoBar = document.getElementById("info-bar");
        infoBar.innerHTML = `(${x}, ${y}, ${z}) &ensp; Empty`
    }

    if (clickCount === 0) {
        clickTime = currentTime;
        clickCount++;
    } else if (currentTime - clickTime < timeLimit && mouseUp.distanceTo(oldClick) < 0.01) {
        mouseUp.x = event.clientX;
        mouseUp.y = event.clientY;
        controls.target.copy(selectMesh.position);
        controls.update();
        clickCount = 0;
    }

    oldClick.copy(mouseUp);
}

function updateVoxel(x, y, z, color) {
    if(color === 0){
        voxels[x][y][z] = null;
    } else {
        voxels[x][y][z] = new THREE.Color(palette[color - 1]);
    }

    let chunkX = Math.floor(x / chunkSize);
    let chunkY = Math.floor(y / chunkSize);
    let chunkZ = Math.floor(z / chunkSize);

    needsUpdate[chunkX][chunkY][chunkZ] = true;
    if(x % chunkSize === 0 && chunkX > 0) needsUpdate[chunkX - 1][chunkY][chunkZ] = true;
    if(y % chunkSize === 0 && chunkY > 0) needsUpdate[chunkX][chunkY - 1][chunkZ] = true;
    if(z % chunkSize === 0 && chunkZ > 0) needsUpdate[chunkX][chunkY][chunkZ - 1] = true;
    if(x % chunkSize === chunkSize - 1 && chunkX < chunks.length - 1) needsUpdate[chunkX + 1][chunkY][chunkZ] = true;
    if(y % chunkSize === chunkSize - 1 && chunkY < chunks[0].length - 1) needsUpdate[chunkX][chunkY + 1][chunkZ] = true;
    if(z % chunkSize === chunkSize - 1 && chunkZ < chunks[0][0].length - 1) needsUpdate[chunkX][chunkY][chunkZ + 1] = true;
}

function changeColor() {
    let colorInputs = document.querySelectorAll('input[name="color"]');
    colorInputs.forEach(input => {
        if(input.checked) {
            selectedColor = input.value;
            selectMesh.material.color.set(selectedColor);
            previewLeftClickMesh.material.color.set(selectedColor);
            previewRightClickMesh.material.color.set(selectedColor);
        }
    });
}

function editVoxel() {
    let x = Math.floor(selectMesh.position.x + size / 2);
    let y = Math.floor(selectMesh.position.y + size / 2);
    let z = Math.floor(selectMesh.position.z + size / 2);
    let voxelvalue = 0;

    for(let i = 0; i < palette.length - 1; i++) {
        if("#" + palette[i].getHexString() === selectedColor) {
            voxelvalue = i + 1;
            break;
        }
    }

    const url = `http://${window.location.hostname}:8000/api/place/temp/draw/${x}/${y}/${z}/${voxelvalue}/client`;
    fetch(url, {method: "POST"})
        .then(response => {
            if (response.ok) {
                if(selectedColor === 'empty'){
                    voxels[x][y][z] = null;
                } else {
                    voxels[x][y][z] = new THREE.Color(selectedColor);
                }

                let chunkX = Math.floor(x / chunkSize);
                let chunkY = Math.floor(y / chunkSize);
                let chunkZ = Math.floor(z / chunkSize);

                updateChunk(chunkX, chunkY, chunkZ);
                if(x % chunkSize === 0 && chunkX > 0) updateChunk(chunkX - 1, chunkY, chunkZ);
                if(y % chunkSize === 0 && chunkY > 0) updateChunk(chunkX, chunkY - 1, chunkZ);
                if(z % chunkSize === 0 && chunkZ > 0) updateChunk(chunkX, chunkY, chunkZ - 1);
                if(x % chunkSize === chunkSize - 1 && chunkX < chunks.length - 1) updateChunk(chunkX + 1, chunkY, chunkZ);
                if(y % chunkSize === chunkSize - 1 && chunkY < chunks[0].length - 1) updateChunk(chunkX, chunkY + 1, chunkZ);
                if(z % chunkSize === chunkSize - 1 && chunkZ < chunks[0][0].length - 1) updateChunk(chunkX, chunkY, chunkZ + 1);
            } else {
                return response.text().then(text => {
                    console.log("Draw error: " + response.status + " " + text);
                });
            }
        });
}

function updateChunk(chunkX, chunkY, chunkZ) {
    let chunk = chunks[chunkX][chunkY][chunkZ];
    scene.remove(chunk);
    let newChunk = generateChunk(chunkX, chunkY, chunkZ);
    chunks[chunkX][chunkY][chunkZ] = newChunk;
    scene.add(newChunk);
}

function updateWhenNeeded(count=0, limit=64) {
    let updated = false;
    if (xUpdate < needsUpdate.length) {
        if (yUpdate < needsUpdate[xUpdate].length) {
            if (zUpdate < needsUpdate[xUpdate][yUpdate].length) {
                if(needsUpdate[xUpdate][yUpdate][zUpdate]) {
                    updateChunk(xUpdate, yUpdate, zUpdate);
                    needsUpdate[xUpdate][yUpdate][zUpdate] = false;
                    console.log("Updated chunk " + xUpdate + ", " + yUpdate + ", " + zUpdate);
                } else {
                    updated = true;
                }
                zUpdate++;
            } else {
                zUpdate = 0;
                yUpdate++;
            }
        } else {
            yUpdate = 0;
            xUpdate++;
        }
    } else {
        xUpdate = 0;
    }

    if(!updated) {
        if(count < limit) {
            updateWhenNeeded(count + 1);
        }
    }
}

function createGradientTexture(colorStops, width=16, height=1024) {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    canvas.width = width;
    canvas.height = height;

    const gradient = context.createLinearGradient(0, 0, 0, height);

    for(let i = 0; i < colorStops[0].length; i++){
        gradient.addColorStop(colorStops[0][i], colorStops[1][i]);
    }

    context.fillStyle = gradient;
    context.fillRect(0, 0, width, height);

    const texture = new THREE.Texture(canvas);
    texture.needsUpdate = true;

    return texture;
}