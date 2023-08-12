<template>
  <NuxtLink :to="'/profile/' + user_id" class="w-full h-full">
    <div class="bg-white border-2 border-black rounded-2xl w-[20rem] h-[20rem] flex flex-col justify-between items-center shadow-custom relative cursor-pointer">
      <VoxelPreview :voxel_id="voxel_id"/>
      <div class="bg-neutral-200 border-black border-t-2 rounded-b-2xl w-full h-10 flex items-center justify-between">
        <div class="flex items-center">
          <div class="bg-blue-300 border-2 border-black rounded-full h-6 w-6 ml-2"></div>
          <p class="ml-2 text-xl">{{ username }}</p>
        </div>
        <TinyButton text="Follow" disabled class="bg-blue-300 hover:bg-blue-400 mr-2"></TinyButton>
      </div>
    </div>
  </NuxtLink>
</template>

<script setup>
const props = defineProps({
  username: {
    type: String,
    default: 'Username'
  },
  user_id: {
    type: String,
    default: '0'
  },
  voxel_id: {
    type: String,
    default: '0'
  },
});
</script>
