<template>
  <div class="bg-blue-300 w-full h-16 flex justify-between items-center border-b-2 border-black">
    <div class="ml-8 w-96">
      <p class="text-3xl">voxplace. [indev]</p>
    </div>
    <div class="relative w-auto h-10 flex justify-center items-center">
      <div class="flex space-x-10 lg:space-x-20 items-center">
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 0; $emit('posts-pressed')">Posts</p>
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 1; $emit('places-pressed')">Places</p>
        <p class="text-2xl cursor-pointer w-20 text-center" @mousedown="selectedItem = 2; $emit('users-pressed')">Users</p>
      </div>
      <div :class="['bg-black w-32 h-0.5 absolute bottom-0', underlinePosition]"></div>
    </div>
    <div class="w-96 flex justify-end space-x-8 mr-8">
      <Button text="Create" :disabled="!isAuth" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
      <Button v-if="!isAuth" @click="$emit('login-clicked')" :text="authButton[authButtonIndex]" class="bg-white hover:bg-neutral-300 w-44"/>
      <Button v-if="isAuth" text="Profile" routerLink="/profile/me" class="bg-white hover:bg-neutral-300 w-44"/>
    </div>
  </div>
</template>

<script setup>
let selectedItem = ref(0)

const positions = ['-left-5', '', '-right-5'];
const authButton = ['Log In', 'Sign Up'];

const props = defineProps({
  authButtonIndex: {
    type: Number,
    default: 0,
  },
  isAuth: {
    type: Boolean,
    default: false,
  },
});

const underlinePosition = computed(() => {
  return positions[selectedItem.value];
});
</script>