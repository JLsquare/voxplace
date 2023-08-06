<template>
  <div @mouseover="hover = true" @mouseleave="hover = false" class="cursor-pointer">
    <svg xmlns="http://www.w3.org/2000/svg" width="25" height="28" viewBox="0 0 25 28" fill="none">
      <path d="M16 15L16 25C16 26 15 27 14 27H11C10 27 9.00004 26 9 25L8.99996 15H2.49999C0.500004 15 0.500003 13 2.49999 11L10.5 3C12 1.5 13 1.5 14.5 3L22.4999 11C24.5 13 24.5 15 22.5 15H16Z" :fill="hover ? hoverColor : color"/>
      <path d="M16 15C16 15 16 24 16 25M16 15C16 15 20.5 15 22.5 15M16 15L16 25M16 15H22.5M16 25C16 26 15 27 14 27M9 25C8.99996 24 8.99996 15 8.99996 15M9 25C9.00004 26 10 27 11 27M9 25L8.99996 15M8.99996 15C8.99996 15 4.49998 15 2.49999 15M8.99996 15H2.49999M2.49999 15C0.500004 15 0.500003 13 2.49999 11M2.49999 11C3.85977 9.64017 8.9999 4.50001 10.5 3M2.49999 11L10.5 3M22.5 15C24.5 15 24.5 13 22.4999 11M22.4999 11C21.1401 9.64016 15.9999 4.50001 14.5 3M22.4999 11L14.5 3M11 27C12 27 13 27 14 27M11 27H14M10.5 3C12 1.5 13 1.5 14.5 3" stroke="black" stroke-width="2" stroke-linejoin="round"/>
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