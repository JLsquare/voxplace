<template>
  <div class="bg-white border-2 border-black rounded-2xl w-full flex justify-between items-center px-4 shadow-custom">
    <input
        type="text"
        v-model="searchQuery"
        placeholder="Search"
        class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none"
    />
    <img src="/search-icon.svg" width="24" @click="search" class="cursor-pointer"/>
  </div>
</template>

<script setup>
let searchQuery = ref('')

const search = () => {
  console.log('Searching for', searchQuery.value)
}
</script>
