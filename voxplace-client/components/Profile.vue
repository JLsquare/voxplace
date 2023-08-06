<template>
  <div v-if="profile" class="bg-white border-2 border-black rounded-2xl shadow-custom w-fit p-8">
    <div class="bg-white border-2 border-black rounded-2xl w-[24rem] h-[24rem] flex justify-between items-center shadow-custom relative cursor-pointer">
      <VoxelPreview :voxel_id="profile.voxel_id"/>
    </div>
    <div class="mt-8">
      <div class="flex mt-4 justify-between">
        <div class="flex">
          <div class="bg-blue-300 border-2 border-black rounded-full w-6 h-6 mt-1"/>
          <p class="text-2xl ml-4">{{ profile.username }}</p>
        </div>
        <TinyButton text="Follow" class="bg-blue-300 hover:bg-blue-400 ml-4"/>
      </div>
      <p class="text-xl mt-4">Joined: {{ unixTimestampToReadableDate(profile.created_at) }}</p>
      <p class="text-xl">Last Online: {{ unixTimestampToReadableDate(profile.last_connected_at) }}</p>
      <p class="text-xl">Followers: 0</p>
      <p class="text-xl mt-4">Level: {{ calculateLevel(profile.xp) }}</p>
      <div class="relative h-6 bg-gray-200 rounded-xl border-2 border-black w-full">
        <div class="absolute h-full bg-blue-300 rounded-xl" :style="{ width: xpProgress(profile.xp) + '%' }"></div>
        <div class="absolute top-0 left-0 right-0 bottom-0 flex justify-center items-center">
          <span>{{ xpDifference(profile.xp) }} / {{ xpRequiredForNextLevel(calculateLevel(profile.xp)) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  user_id: {
    type: String,
    required: true
  }
});

let profile = ref(null);

onMounted(() => {
  getProfile();
});

async function getProfile() {
  const token = localStorage.getItem('token');

  let res = await fetch(`http://localhost:8000/api/user/profile/${props.user_id}`, {
    headers: {
      'Authorization': token
    }
  });
  if(res.ok) {
    let data = await res.json();
    console.log(data);
    profile.value = data;
  } else {
    let error = await res.text();
    console.error(error);
  }
}

function unixTimestampToReadableDate(timestamp) {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString() + " " + date.toLocaleTimeString();
}

function calculateLevel(xp) {
  return Math.floor(Math.pow(xp, 1/1.5) / 10);
}

function xpRequiredForLevel(level) {
  return Math.floor(Math.pow(level * 10, 1.5));
}

function xpRequiredForNextLevel(level) {
  return xpRequiredForLevel(level + 1) - xpRequiredForLevel(level);
}

function xpDifference(currentXp) {
  const level = calculateLevel(currentXp);
  const xpForLevel = xpRequiredForLevel(level);
  return currentXp - xpForLevel;
}

function xpProgress(currentXp) {
  const level = calculateLevel(currentXp);
  const xpForThisLevel = xpRequiredForLevel(level);
  const xpForNextLevel = xpRequiredForLevel(level + 1);
  return ((currentXp - xpForThisLevel) / (xpForNextLevel - xpForThisLevel)) * 100;
}
</script>