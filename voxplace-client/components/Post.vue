<template>
  <div class="bg-white border-2 border-black rounded-2xl w-[37rem] h-96 flex flex-col mx-4 shadow-custom relative">
    <div class="w-full h-full">
      <VoxelPreview :voxel_id="post.voxel_id"/>
    </div>
    <div class="bg-neutral-200 border-black border-t-2 rounded-b-2xl w-full h-10 flex items-center justify-between">
      <div class="flex items-center">
        <div class="bg-blue-300 border-2 border-black rounded-full w-6 h-6 ml-2"/>
        <p class="text-xl ml-2">{{ post.title }}</p>
      </div>
      <div class="flex item-center">
        <CommentButton class ="mr-4 mt-0.5"/>
        <ArrowButton class="mr-2 mb-0.5 transform rotate-180"/>
        <p class="text-xl mr-2 mb-1">{{ post.votes }}</p>
        <ArrowButton class="mr-2"/>
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  post: {
    type: Object,
    required: true
  }
})
</script>
