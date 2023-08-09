<template>
  <div class="m-0 flex flex-col justify-center items-center h-screen font-roboto">
    <PlaceTopBar :text="infoBarText" class="absolute top-0 left-0"/>
    <canvas id="canvas" />
    <div class="flex absolute bottom-0 items-center justify-center mb-4 h-24 space-x-8">
      <PlacePalette
          v-if="selectMesh != null"
          :x="Math.floor(selectMesh.position.x + size / 2)"
          :y="Math.floor(selectMesh.position.y + size / 2)"
          :z="Math.floor(selectMesh.position.z + size / 2)"
          @drawed="updateInfoBar"
      />
    </div>
  </div>
</template>

<script setup>
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import {GLTFLoader} from 'three/addons/loaders/GLTFLoader.js';

const route = useRoute();
let size = 0;
let chunkSize = 0;
const moveSpeed = 0.5;
let scene;
let camera;
let renderer;
let controls;
let loader;
let infoBarText = ref('(0, 0, 0)   Empty / Server');

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
let needsUpdate = [];
let xUpdate = 0;
let yUpdate = 0;
let zUpdate = 0;
let socket;

onMounted(() => {
  init();
})

async function init() {
  await initPalette();
  await initVoxelData();
  initSocket();
  addEventListeners();
}

function initSocket() {
  socket = new WebSocket(`ws://${window.location.hostname}:8000/api/place/ws/${route.params.id}`);

  socket.onopen = () => {
    console.log('[open] Connection established');
  };

  socket.onerror = (error) => {
    console.log(`[error] ${error.message}`);
  }

  socket.onmessage = (event) => {
    let data = JSON.parse(event.data);
    if (data.type === 'update') {
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

function initScene() {
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0xffffff);
  camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 2000);
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    canvas: document.querySelector('#canvas'),
  });
  controls = new OrbitControls(camera, renderer.domElement);
  loader = new GLTFLoader();

  controls.maxDistance = 512;
  controls.minDistance = 2;

  const planeGeometry = new THREE.PlaneGeometry(size, size);
  planeGeometry.applyMatrix4(new THREE.Matrix4().makeRotationX(-Math.PI / 2));

  const invertedBoxGeometry = new THREE.BoxGeometry(size, size, size);
  invertedBoxGeometry.applyMatrix4(new THREE.Matrix4().makeScale(-1, -1, -1));

  const borderMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, transparent: true, opacity: 0.05 });
  const borderMesh = new THREE.Mesh(invertedBoxGeometry, borderMaterial);
  borderMesh.position.set(-0.5, -0.5, -0.5);
  borderMesh.raycast = () => [];
  scene.add(borderMesh);

  const floorMaterial = new THREE.MeshBasicMaterial({ color: 0x000000 });
  const floorMesh = new THREE.Mesh(planeGeometry, floorMaterial);
  floorMesh.position.set(-0.5, -size / 2 - 0.5, -0.5);
  floorMesh.visible = false;
  scene.add(floorMesh);

  const edgesGeometry = new THREE.EdgesGeometry(invertedBoxGeometry);
  const edgesMaterial = new THREE.LineBasicMaterial({ color: 0x000000 });
  const edgesMesh = new THREE.LineSegments(edgesGeometry, edgesMaterial);
  edgesMesh.position.set(-0.5, -0.5, -0.5);
  scene.add(edgesMesh);

  edgesMesh.raycast = () => [];

  loader.load('/selected.glb', (gltf) => {
    const selectMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, depthTest: false, transparent: true });
    const selectGeometry = gltf.scene.children[0].geometry;
    const previewLeftClickMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, transparent: true, opacity: 1, depthTest: false });
    const previewRightClickMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, transparent: true, opacity: 0.25, depthTest: false });
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

  camera.position.set(size / 2, size / 2, size / 2);
}

async function initPalette() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/place/palette/${route.params.id}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    const message = await response.text();
    console.log(message);
  } else {
    palette = await response.json();
  }
}

async function initVoxelData() {
  await fetch(`http://${window.location.hostname}:8000/api/place/all/${route.params.id}`)
      .then(response => response.arrayBuffer())
      .then(data => {
        const bytes = new Uint8Array(data);
        size = Math.cbrt(bytes.length);
        if (size < 16 || size % 16 !== 0) {
          chunkSize = size;
        } else if (size % 16 === 0) {
          chunkSize = 16;
        }
        for(let x = 0; x < size; x++) {
          voxels[x] = [];
          for(let y = 0; y < size; y++) {
            voxels[x][y] = [];
            for(let z = 0; z < size; z++) {
              const value = bytes[x * size * size + y * size + z];
              if(value > 0) {
                voxels[x][y][z] = value;
              } else {
                voxels[x][y][z] = 0;
              }
            }
          }
        }
        initScene();
        initChunks();
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
  animate();
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
        if (voxel > 0) {
          let leftEmpty = x + dx - 1 < 0 || voxels[x + dx - 1][y + dy][z + dz] === 0;
          let rightEmpty = x + dx + 1 >= size || voxels[x + dx + 1][y + dy][z + dz] === 0;
          let bottomEmpty = y + dy - 1 < 0 || voxels[x + dx][y + dy - 1][z + dz] === 0;
          let topEmpty = y + dy + 1 >= size || voxels[x + dx][y + dy + 1][z + dz] === 0;
          let frontEmpty = z + dz - 1 < 0 || voxels[x + dx][y + dy][z + dz - 1] === 0;
          let backEmpty = z + dz + 1 >= size || voxels[x + dx][y + dy][z + dz + 1] === 0;

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

          const faceNormalDirections = [0, 1, 2, 3, 4, 5];
          const shadingFactors = [0.5, 0.9, 0.8, 0.7, 0.6, 1.0];

          for(let i = 0; i < 6; i++) {
            if([leftEmpty, rightEmpty, bottomEmpty, topEmpty, frontEmpty, backEmpty][i]) {
              let color = new THREE.Color(palette[voxel - 1]);

              color.r *= shadingFactors[faceNormalDirections[i]];
              color.g *= shadingFactors[faceNormalDirections[i]];
              color.b *= shadingFactors[faceNormalDirections[i]];

              chunkColors.push(color.r, color.g, color.b);
              chunkColors.push(color.r, color.g, color.b);
              chunkColors.push(color.r, color.g, color.b);
              chunkColors.push(color.r, color.g, color.b);
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
  requestAnimationFrame(animate);
  const frustum = new THREE.Frustum();
  frustum.setFromProjectionMatrix(new THREE.Matrix4().multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse));
  for(let x = 0; x < size / chunkSize; x++) {
    for(let y = 0; y < size / chunkSize; y++) {
      for(let z = 0; z < size / chunkSize; z++) {
        if(chunks[x][y][z]) {
          chunks[x][y][z].visible = frustum.intersectsObject(chunks[x][y][z]);
        }
      }
    }
  }
  updateWhenNeeded();
  controls.update();
  renderer.render(scene, camera);
}

function addEventListeners() {
  document.addEventListener('keydown', handleKeyDown, false);
  document.addEventListener('mousemove', handleMouseMove, false);
  document.addEventListener('mousedown', handleMouseDown, false);
  document.addEventListener('mouseup', handleMouseUp, false);
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

      let x = Math.floor(leftClickPosition.x + (size / 2));
      let y = Math.floor(leftClickPosition.y + (size / 2));
      let z = Math.floor(leftClickPosition.z + (size / 2));

      if(leftClickPosition.y <= -size / 2 && voxels[x][y][z] === 0) {
        rightClickPosition.y = -size / 2;
      }

      leftClickPosition.add(new THREE.Vector3(0.5, 0.5, 0));
      if(previewLeftClickMesh) {
        previewLeftClickMesh.position.copy(leftClickPosition);
      }

      rightClickPosition.add(new THREE.Vector3(0.5, 0.5, 0));
      if(previewRightClickMesh) {
        previewRightClickMesh.position.copy(rightClickPosition);
      }
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

async function handleMouseUp(event) {
  if (event.target.id !== 'canvas') {
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
    if (event.button === 0) {
      selectMesh.position.copy(previewLeftClickMesh.position);
    } else if (event.button === 2) {
      selectMesh.position.copy(previewRightClickMesh.position);
    }

    let x = Math.floor(selectMesh.position.x + size / 2 - 0.5);
    let y = Math.floor(selectMesh.position.y + size / 2 - 0.5);
    let z = Math.round(selectMesh.position.z + size / 2 - 0.5);

    let usernameRequest = {
      x: x,
      y: y,
      z: z
    }

    const response = await fetch(`http://${window.location.hostname}:8000/api/place/username/${route.params.id}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(usernameRequest)
    });

    const username = await response.json();
    updateInfoBar(username);
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

function updateInfoBar(username){
  let x = Math.floor(selectMesh.position.x + size / 2 - 0.5);
  let y = Math.floor(selectMesh.position.y + size / 2 - 0.5);
  let z = Math.round(selectMesh.position.z + size / 2 - 0.5);

  infoBarText.value = `(${x}, ${y}, ${z})   ${username}`
}

function updateVoxel(x, y, z, color) {
  voxels[x][y][z] = color;

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
</script>