<template>
  <div class="bg-white border-2 border-black rounded-2xl w-[20rem] h-[20rem] flex flex-col justify-between items-center shadow-custom relative">
    <NuxtLink :to="`/editor/${voxel_id}`" class="w-full h-full">
      <VoxelPreview :voxel_id="voxel_id" />
    </NuxtLink>
    <div class="bg-neutral-200 border-black border-t-2 px-2 rounded-b-2xl w-full h-10 flex items-center justify-between z-10">
      <div class="flex items-center">
        <p class="text-xl ml-2">{{ name }}</p>
      </div>
      <div class="flex item-center">
        <TinyButton class ="bg-blue-300 hover:bg-blue-400" text="Post" />
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  voxel_id: {
    type: String,
    default: '0'
  },
  name: {
    type: String,
    default: 'Voxel'
  }
})
</script>
