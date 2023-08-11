<template>
  <div class="flex items-center">
    <div v-if="opened" class="bg-white py-2 px-4 overflow-hidden rounded-2xl border-2 border-black shadow-custom grid grid-rows-2 grid-flow-col gap-2">
      <div
          v-for="(color, index) in palette"
          :key="index"
          @click="selectedColor = color; sendVoxel(); opened = false"
          class="w-8 h-8 rounded-lg cursor-pointer hover:border-2 border border-black"
          :style="{ backgroundColor: color }"
      >
      </div>
      <div
          @click="selectedColor = 'remove'; sendVoxel(); opened = false"
          class="w-8 h-8 rounded-lg cursor-pointer hover:border-2 border border-black"
          style="background-image: url(/remove.png); background-size: cover"></div>
      <CloseButton class="order-last p-0.5" size="28"/>
    </div>
    <Button text="Submit" id="edit-voxel-btn" v-if="!opened && !isCooldownActive" @click="opened = true" class="bg-white hover:bg-neutral-200"/>
    <Button disabled :text="remainingCooldown + ' seconds'" v-if="isCooldownActive" class="bg-white hover:bg-neutral-200"/>
  </div>
</template>

<script setup>
const route = useRoute();

let palette = ref([]);
let selectedColor = ref('');
let opened = ref(false);
let cooldown = ref(60);
let currentTime = ref(Date.now());

const emit = defineEmits(['drawed']);

onMounted(() => {
  getPalette();
  getCooldown();
  setInterval(() => {
    currentTime.value = Date.now();
  }, 1000);
});

async function getCooldown() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/place/cooldown/${route.params.id}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': localStorage.getItem('token')
    },
  });

  if (!response.ok) {
    const message = await response.text();
    console.log(message);
  } else {
    cooldown.value = await response.json();
  }
}

async function getPalette() {
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
    palette.value = await response.json();
  }
}

const props = defineProps({
  x: {
    type: Number,
    required: true
  },
  y: {
    type: Number,
    required: true
  },
  z: {
    type: Number,
    required: true
  },
})

async function sendVoxel() {
  let colorIndex = selectedColor.value === 'remove' ? 0 : palette.value.indexOf(selectedColor.value) + 1;

  let drawRequest = {
    x: props.x,
    y: props.y,
    z: props.z,
    color: colorIndex,
  }

  fetch(`http://${window.location.hostname}:8000/api/place/draw/${route.params.id}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': localStorage.getItem('token'),
    },
    body: JSON.stringify(drawRequest)
  })
      .then(async response => {
        if (response.ok) {
          let responseJson = await response.json();
          let username = responseJson.username;
          cooldown.value = responseJson.cooldown;
          emit('drawed', username);
        } else {
          console.log(await response.text());
        }
      })
}

let isCooldownActive = computed(() => {
  let cooldownMillis = cooldown.value * 1000;
  return cooldownMillis > currentTime.value;
});

let remainingCooldown = computed(() => {
  let cooldownMillis = cooldown.value * 1000;
  if(cooldownMillis > currentTime.value) {
    return ((cooldownMillis - currentTime.value)/1000).toFixed(0);
  }
  return 0;
});

</script>
