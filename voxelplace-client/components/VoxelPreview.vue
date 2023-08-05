<template>
  <div id="parent" class="rounded-2xl h-full w-full">
    <canvas id="canvas" class="rounded-2xl" />
  </div>
</template>

<script setup>
const props = defineProps({
  voxel_id: {
    type: String,
    default: '0'
  }
})

import * as THREE from 'three';

const size = 128;
const chunkSize = 16;
let scene;
let camera;
let renderer;
let cameraAngle = 0;
const cameraSpeed = 0.01;

const invertedBoxGeometry = new THREE.BoxGeometry(size, size, size);
invertedBoxGeometry.applyMatrix4(new THREE.Matrix4().makeScale(-1, -1, -1));
const planeGeometry = new THREE.PlaneGeometry(size, size);
planeGeometry.applyMatrix4(new THREE.Matrix4().makeRotationX(-Math.PI / 2));

let voxels = [];
let palette = [];
let chunks = [];

const raycaster = new THREE.Raycaster();
let needsUpdate = [];

onMounted(() => {
  init();
})

async function init() {
  initScene();
  await initPalette();
  await initVoxelData();
}

function initScene() {
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0xffffff);
  let canvas = document.querySelector('#canvas')
  let parent = document.querySelector('#parent')
  camera = new THREE.PerspectiveCamera(75, parent.offsetWidth / parent.offsetHeight, 0.1, 2000);
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    canvas: canvas,
  });

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

  renderer.setSize(parent.offsetWidth, parent.offsetHeight);

  camera.position.set(0, 0, size);
  camera.lookAt(0, 0, 0);
}

async function initPalette() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/voxel/palette/${props.voxel_id}`, {
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
  await fetch(`http://${window.location.hostname}:8000/api/voxel/all/${props.voxel_id}`)
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
                voxels[x][y][z] = value;
              } else {
                voxels[x][y][z] = 0;
              }
            }
          }
        }
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

          for(let i = 0; i < 6; i++) {
            if([leftEmpty, rightEmpty, bottomEmpty, topEmpty, frontEmpty, backEmpty][i]) {
              let color = new THREE.Color(palette[voxel - 1])
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

  cameraAngle += cameraSpeed;
  const radius = size * 1.25;
  camera.position.x = radius * Math.sin(cameraAngle);
  camera.position.z = radius * Math.cos(cameraAngle);
  camera.lookAt(0, 0, 0);

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
  renderer.render(scene, camera);
}
</script>