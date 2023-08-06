<template>
  <div class="bg-blue-300 w-full h-16 flex justify-between items-center border-b-2 border-black">
    <div class="ml-8 w-96">
      <p class="text-3xl">voxplace. [indev] admin</p>
    </div>
    <div v-show="isAuth" class="relative w-auto h-10 flex justify-center items-center">
      <div class="flex space-x-10 lg:space-x-20 items-center">
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 0; $emit('voxels-pressed')">Voxels</p>
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 1; $emit('places-pressed')">Places</p>
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 2; $emit('users-pressed')">Users</p>
      </div>
      <div :class="['bg-black w-32 h-0.5 absolute bottom-0', underlinePosition]"></div>
    </div>
    <div class="w-96 flex justify-end space-x-8 mr-8">
      <Button v-if="!isAuth" @click="$emit('login-clicked')" text="Log In" class="bg-white hover:bg-neutral-300"/>
      <Button v-if="isAuth" @click="$emit('logout-clicked')" text="Log Out" class="bg-white hover:bg-neutral-300"/>
    </div>
  </div>
</template>

<script setup>
let selectedItem = ref(0)
const positions = ['-left-5', '', '-right-5'];

const props = defineProps({
  isAuth: {
    type: Boolean,
    default: false,
  },
});

const underlinePosition = computed(() => {
  return positions[selectedItem.value];
});
</script>