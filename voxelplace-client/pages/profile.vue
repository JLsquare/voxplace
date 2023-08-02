<template>
  <div class="h-screen font-roboto">
    <ProfileTopBar @edit-pressed="toggleLoginEditor()"/>
    <Profile class="absolute left-0 top-0 ml-16 mt-32"/>
    <div class="flex flex-col items-center">
      <div class="flex justify-center items-center mt-4 w-[37rem] flex-col z-10">
        <SearchBar />
        <div class="flex space-x-8 justify-center mt-4">
          <Button text="Trending" class="bg-white hover:bg-neutral-200"/>
          <Button text="New" class="bg-white hover:bg-neutral-200"/>
        </div>
      </div>
      <div class="h-screen absolute top-0 pt-[10.5rem]">
        <Voxels/>
      </div>
    </div>
    <ProfileEditor v-if="showLoginEditor" class="absolute right-0 top-0 mr-16 mt-32"/>
  </div>
</template>

<script setup>
let showLoginEditor = ref(false);

function toggleLoginEditor() {
  showLoginEditor.value = !showLoginEditor.value;
}
</script>