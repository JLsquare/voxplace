<template>
  <div class="bg-blue-300 w-full h-16 flex justify-between items-center border-b-2 border-black">
    <div class="ml-8 w-96">
      <p class="text-3xl">voxelplace.</p>
    </div>
    <div class="relative w-auto h-10 flex justify-center items-center">

    </div>
    <div class="w-96 flex justify-end space-x-8 mr-8">
      <Button text="Log Out" @click="logout" routerLink="/" class="xl:block hidden bg-white hover:bg-neutral-300"/>
      <Button text="Edit Profile" @click="$emit('edit-pressed')" class="xl:block hidden bg-white hover:bg-neutral-300"/>
      <Button text="Home" routerLink="/" class="xl:block hidden bg-white hover:bg-neutral-300"/>
    </div>
  </div>
</template>

<script setup>
function logout(){
  localStorage.removeItem('token');
}
</script>