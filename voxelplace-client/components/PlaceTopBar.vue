<template>
  <div class="bg-blue-300 w-full h-16 flex justify-between items-center border-b-2 border-black">
    <div class="ml-8 w-96">
      <p class="text-3xl">voxelplace.</p>
    </div>
    <div class="relative w-auto h-10 flex justify-center items-center">
      <div class="border-2 border-black rounded-2xl justify-center items-center px-4 shadow-custom w-72 text-center bg-white">
        <p id="info-bar" class="text-lg">(0, 0, 0)   Empty / Server</p>
      </div>
    </div>
    <div class="w-96 flex justify-end space-x-8 mr-8">
      <Button text="Home" routerLink="/" class="xl:block hidden bg-white hover:bg-neutral-300"/>
    </div>
  </div>
</template>

<script setup>

</script>