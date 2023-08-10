<template>
  <div class="bg-blue-300 w-full h-16 flex justify-between items-center border-b-2 border-black">
    <div class="ml-8 w-96">
      <p class="text-3xl">voxplace. [indev]</p>
    </div>
    <div class="relative w-auto h-10 flex justify-center items-center">
      <div class="flex space-x-10 lg:space-x-20 items-center">
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 0; $emit('menu-pressed', 0)">Profile</p>
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 1; $emit('menu-pressed', 1)">Voxels</p>
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 2; $emit('menu-pressed', 2)">Soon</p>
      </div>
      <div :class="['bg-black w-32 h-0.5 absolute bottom-0', underlinePosition]"></div>
    </div>
    <div class="w-96 flex justify-end space-x-8 mr-8">
      <Button text="Log Out" v-if="isMe" @click="logout" routerLink="/" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
      <Button text="Profile" v-if="!isMe" routerLink="/profile/me" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
      <Button text="Log In" v-if="!isMe" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
      <Button text="Home" routerLink="/" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
    </div>
  </div>
</template>

<script setup>
let selectedItem = ref(0)
const positions = ['-left-5', '', '-right-5'];

const underlinePosition = computed(() => {
  return positions[selectedItem.value];
});

function logout(){
  localStorage.removeItem('token');
}

const props = defineProps({
  isMe: {
    type: Boolean,
    default: false
  }
});
</script>