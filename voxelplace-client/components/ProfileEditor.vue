<template>
  <div class="bg-white border-2 border-black rounded-2xl shadow-custom w-fit p-8">
    <div class="bg-white border-2 border-black rounded-2xl w-[24rem] h-[24rem] flex justify-between items-center shadow-custom relative cursor-pointer">
      <div class="bg-neutral-200 border-black border-t-2 rounded-b-2xl w-full h-10 absolute bottom-0 flex items-center justify-between">
        <p class="ml-8 text-xl">Go to editor</p>
      </div>
    </div>
    <div class="mt-8 flex-col">
      <p class="text-xl w-full">Username :</p>
      <input type="text" placeholder="Username" class="px-2 border-2 border-black rounded-xl w-full h-8"/>
      <p class="text-xl w-full mt-4">Email :</p>
      <input type="text" placeholder="Email" class="px-2 border-2 border-black rounded-xl w-full h-8"/>
      <p class="text-xl w-full mt-4">Password :</p>
      <input type="text" placeholder="Password" class="px-2 border-2 border-black rounded-xl w-full h-8"/>
      <div class="flex justify-between mt-8">
        <Button text="Cancel" class="bg-white hover:bg-neutral-300"/>
        <Button text="Save" class="bg-blue-300 hover:bg-blue-400"/>
      </div>
    </div>
  </div>
</template>

<script setup>

</script>