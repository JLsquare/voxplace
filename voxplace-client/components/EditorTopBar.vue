<template>
  <div class="bg-blue-300 w-full h-16 flex justify-between items-center border-b-2 border-black">
    <div class="ml-8 w-96">
      <p class="text-3xl">voxplace. [indev]</p>
    </div>
    <div class="w-96 flex justify-end space-x-8 mr-8">
      <Button text="Back" :routerLink="back" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
      <Button text="Home" routerLink="/" class="xl:block hidden bg-white hover:bg-neutral-300 w-44"/>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  back: {
    type: String,
    default: '/profile/me'
  }
});
</script>