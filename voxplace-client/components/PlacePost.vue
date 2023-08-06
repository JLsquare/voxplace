<template>
  <div class="bg-white border-2 border-black rounded-2xl w-[30rem] h-[20rem] flex flex-col justify-between items-center shadow-custom relative">
    <NuxtLink :to="`/place/${place_id}`" class="w-full h-full">
      <VoxelPreview :voxel_id="voxel_id" />
    </NuxtLink>
    <div class="bg-neutral-200 border-black border-t-2 rounded-b-2xl w-full h-10 flex items-center justify-between z-10">
      <div class="flex items-center">
        <div :class="['border-2 border-black rounded-full w-6 h-6 ml-2', online ? 'bg-blue-300' : 'bg-neutral-300']"/>
        <p class="text-xl ml-2">{{ name }}</p>
      </div>
      <div class="flex item-center">
        <CommentButton class ="mr-4 mt-0.5"/>
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  online: {
    type: Boolean,
    default: false
  },
  name: {
    type: String,
    default: 'Place'
  },
  place_id: {
    type: String,
    default: '0'
  },
  voxel_id: {
    type: String,
    default: '0'
  }
})
</script>
