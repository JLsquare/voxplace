<template>
  <div class="bg-white border-2 border-black rounded-2xl w-[24rem] h-[24rem] flex justify-between items-center shadow-custom relative">
    <NuxtLink :to="`/place/${name}`" class="w-full h-full absolute top-0 left-0"/>
    <div class="bg-neutral-200 border-black border-t-2 rounded-b-2xl w-full h-10 absolute bottom-0 flex items-center justify-between">
      <div class="flex items-center">
        <div :class="['border-2 border-black rounded-full w-6 h-6 ml-2', online ? 'bg-blue-300' : 'bg-neutral-300']"/>
        <p class="text-xl ml-2">{{ name }}</p>
      </div>
      <div class="flex item-center">
        <TinyButton text="Edit" class="mr-4 bg-blue-300"/>
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
  }
})
</script>
