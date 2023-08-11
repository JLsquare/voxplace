<template>
  <div ref="parentRef" class="rounded-2xl h-full w-full">
    <canvas ref="canvasRef" class="rounded-2xl" />
  </div>
</template>

<script setup>
import * as THREE from 'three';

const props = defineProps({
  voxel_id: {
    type: String,
    default: '0'
  }
})

const parentRef = ref(null);
const canvasRef = ref(null);

let size;
let chunkSize;
let scene;
let camera;
let renderer;
let cameraAngle = 0;
let cameraSpeed = 0.5;
let lastTime = 0;
let voxels = [];
let palette = [];
let chunks = [];
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
  let canvas = canvasRef.value;
  let parent = parentRef.value;
  camera = new THREE.PerspectiveCamera(75, parent.offsetWidth / parent.offsetHeight, 0.1, 2000);
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    canvas: canvas,
  });

  renderer.setSize(parent.offsetWidth, parent.offsetHeight);
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
  await fetch(`http://${window.location.hostname}:8000/api/voxel/all/${props.voxel_id}`)
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

function animate(time = 0) {
  requestAnimationFrame(animate);

  const delta = (time - lastTime) / 1000;
  lastTime = time;

  cameraAngle += cameraSpeed * delta;
  const radius = size * 1.35;
  camera.position.x = radius * Math.sin(cameraAngle) - 0.5;
  camera.position.z = radius * Math.cos(cameraAngle) - 0.5;
  camera.position.y = size * 0.35;
  camera.lookAt(-0.5, -1, -0.5);

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