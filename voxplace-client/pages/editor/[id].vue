<template>
  <div class="m-0 flex flex-col justify-center items-center h-screen font-roboto">
    <EditorTopBar />
    <div class="flex w-screen h-full px-8 py-4 space-x-8">
      <div class="h-full w-1/3">
        <EditorPalette @selection-changed="selectionChanged" @save="saveVoxelData"/>
      </div>
      <div class="border-2 border-black rounded-2xl h-full w-2/3 shadow-custom">
        <div ref="parentRef" class="h-full w-full rounded-2xl">
          <canvas ref="canvasRef" class="rounded-2xl" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import {GLTFLoader} from 'three/addons/loaders/GLTFLoader.js';
import pako from 'pako';

let selectedColor = ref(-1);
let selectedTool = ref(-1);
let selectedAction = ref(-1);

const route = useRoute();
let size = 0;
let chunkSize = 0;
const moveSpeed = 0.5;
let scene;
let camera;
let renderer;
let controls;
let loader;

const parentRef = ref(null);
const canvasRef = ref(null);

let voxels = [];
let palette = [];
let selectMesh = null;
let previewMesh = null;
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

onMounted(() => {
  init();
})

function selectionChanged(selections){
  selectedColor.value = selections.color;
  selectedTool.value = selections.tool;
  selectedAction.value = selections.action;
}

async function init() {
  await initPalette();
  await initVoxelData();
  initScene();
  initChunks();
  addEventListeners();
}

function initScene() {
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0xffffff);
  let canvas = canvasRef.value;
  let parent = parentRef.value;
  camera = new THREE.PerspectiveCamera(75, parent.offsetWidth / parent.offsetHeight, 0.1, 2000);
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    canvas: canvas,
  });
  controls = new OrbitControls(camera, renderer.domElement);
  loader = new GLTFLoader();

  controls.maxDistance = 512;
  controls.minDistance = 2;

  const invertedBoxGeometry = new THREE.BoxGeometry(size, size, size);
  invertedBoxGeometry.applyMatrix4(new THREE.Matrix4().makeScale(-1, -1, -1));

  const borderMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, transparent: true, opacity: 0.05 });
  const borderMesh = new THREE.Mesh(invertedBoxGeometry, borderMaterial);
  borderMesh.position.set(-0.5, -0.5, -0.5);
  scene.add(borderMesh);

  const edgesGeometry = new THREE.EdgesGeometry(invertedBoxGeometry);
  const edgesMaterial = new THREE.LineBasicMaterial({ color: 0x000000 });
  const edgesMesh = new THREE.LineSegments(edgesGeometry, edgesMaterial);
  edgesMesh.position.set(-0.5, -0.5, -0.5);
  scene.add(edgesMesh);

  edgesMesh.raycast = () => [];

  loader.load('/selected.glb', (gltf) => {
    const selectMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, depthTest: false, transparent: true, opacity: 0.5 });
    const selectGeometry = gltf.scene.children[0].geometry;
    const previewLeftClickMaterial = new THREE.MeshBasicMaterial({ color: 0x000000, transparent: true, opacity: 0.25, depthTest: false });
    selectMesh = new THREE.Mesh(selectGeometry, selectMaterial);
    selectMesh.scale.set(0.05, 0.05, 0.05);
    selectMesh.renderOrder = 2;
    scene.add(selectMesh);
    selectMesh.visible = false;
    previewMesh = new THREE.Mesh(selectGeometry, previewLeftClickMaterial);
    previewMesh.scale.set(0.05, 0.05, 0.05);
    previewMesh.renderOrder = 3;
    scene.add(previewMesh);
  });

  renderer.setSize(parent.offsetWidth, parent.offsetHeight);

  camera.position.set(size / 2, size / 2, size / 2);
}

async function initPalette() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/palette/get/0`, {
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
  await fetch(`http://${window.location.hostname}:8000/api/voxel/all/${route.params.id}`)
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
      });
}

async function saveVoxelData() {
  let bytes = new Uint8Array(size * size * size);
  for(let x = 0; x < size; x++) {
    for(let y = 0; y < size; y++) {
      for(let z = 0; z < size; z++) {
        bytes[x * size * size + y * size + z] = voxels[x][y][z];
      }
    }
  }

  const compressedBytes = pako.gzip(bytes);

  const response = await fetch(`http://${window.location.hostname}:8000/api/voxel/save/${route.params.id}`, {
    method: 'POST',
    headers: {
      'Content-Encoding': 'gzip',
      'Content-Type': 'application/octet-stream',
    },
    body: compressedBytes,
  });
  if(response.ok){
    console.log(await response.json);
  } else {
    console.log(await response.text());
  }
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
  let parent = parentRef.value;
  if(!parent) return;
  let rect = parent.getBoundingClientRect();

  mouse.x = ((event.clientX - rect.left) / parent.offsetWidth) * 2 - 1;
  mouse.y = -((event.clientY - rect.top) / parent.offsetHeight) * 2 + 1;

  mouseUp.x = event.clientX;
  mouseUp.y = event.clientY;

  raycaster.setFromCamera(mouse, camera);
  const intersects = raycaster.intersectObjects(scene.children, true);

  if (intersects.length > 0) {
    if(intersects[0].object !== selectMesh && intersects[0].object !== previewMesh) {
      const selectedVoxel = intersects[0];
      let position;
      if(selectedAction.value === 0) {
        position = new THREE.Vector3(
            Math.round(selectedVoxel.point.x + selectedVoxel.face.normal.x / 2),
            Math.round(selectedVoxel.point.y + selectedVoxel.face.normal.y / 2),
            Math.round(selectedVoxel.point.z + selectedVoxel.face.normal.z / 2)
        );
      } else {
        position = new THREE.Vector3(
            Math.round(selectedVoxel.point.x - selectedVoxel.face.normal.x / 2),
            Math.round(selectedVoxel.point.y - selectedVoxel.face.normal.y / 2),
            Math.round(selectedVoxel.point.z - selectedVoxel.face.normal.z / 2)
        );
      }

      if (position.x < -size / 2) { position.x = -size / 2; }
      if (position.x > size / 2 - 1) { position.x = size / 2 - 1; }
      if (position.y < -size / 2) { position.y = -size / 2; }
      if (position.y > size / 2 - 1) { position.y = size / 2 - 1; }
      if (position.z < -size / 2) { position.z = -size / 2; }
      if (position.z > size / 2 - 1) { position.z = size / 2 - 1; }

      let x = Math.floor(position.x + (size / 2));
      let y = Math.floor(position.y + (size / 2));
      let z = Math.floor(position.z + (size / 2));

      position.add(new THREE.Vector3(0.5, 0.5, 0));
      if(previewMesh) {
        previewMesh.position.copy(position);
      }
    }
  }
}

function handleMouseDown(event) {
  mouseDown.x = event.clientX;
  mouseDown.y = event.clientY;
}

async function handleMouseUp(event) {
  currentTime = new Date().getTime();

  if (clickCount > 0 && currentTime - clickTime > timeLimit) {
    clickCount = 0;
  }

  if (mouseDown.distanceTo(mouseUp) < 0.01) {
    if (selectMesh.visible === false) {
      selectMesh.visible = true;
    }
    selectMesh.position.copy(previewMesh.position);
    if(selectedTool.value === 0){
      pointTool();
    }
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

function pointTool() {
  if (selectMesh.visible === true) {
    let x = Math.floor(selectMesh.position.x + (size / 2));
    let y = Math.floor(selectMesh.position.y + (size / 2));
    let z = Math.floor(selectMesh.position.z + (size / 2));
    console.log(x, y, z);

    if (x >= 0 && x < size && y >= 0 && y < size && z >= 0 && z < size) {
      if (selectedAction.value === 0) {
        if (voxels[x][y][z] !== selectedColor.value + 1) {
          updateVoxel(x, y, z, selectedColor.value + 1);
        }
      } else if (selectedAction.value === 1) {
        if (voxels[x][y][z] !== 0) {
          updateVoxel(x, y, z, 0);
        }
      }
    }
  }
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