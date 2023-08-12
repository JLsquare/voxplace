<template>
  <div @mouseover="hover = true" @mouseleave="hover = false" class="cursor-pointer">
    <svg width="27" height="27" viewBox="0 0 27 27" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M26 13.5L1 26V16.625L13.5 13.5L1 10.375V1L26 13.5Z" :fill="hover ? hoverColor : color" stroke="black" stroke-width="2" stroke-linejoin="round"/>
    </svg>
  </div>
</template>

<script setup>
const hover = ref(false);

const props = defineProps({
  color: {
    type: String,
    default: 'white'
  },
  hoverColor: {
    type: String,
    default: '#e5e5e5'
  },
  size: {
    type: String,
    default: '24'
  }
});
</script>