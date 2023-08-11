<template>
  <div @mouseover="hover = true" @mouseleave="hover = false" class="cursor-pointer">
    <svg xmlns="http://www.w3.org/2000/svg" width="34" height="28" viewBox="0 0 34 28" fill="none">
      <path d="M1 5C1 3 2.99999 1 5 1H29C31 1 33 3 33 5V17C33 19 31 21 29 21H17L13 25C11 27 9 27 9 25V21H5C3 21 1 19 1 17V5Z" :fill="hover ? hoverColor : color"/>
      <path d="M17 21C17 21 27 21 29 21M17 21H29M17 21C17 21 15 23 13 25M17 21L13 25M9 21C9 21 7 21 5 21M9 21H5M9 21C9 21 9 23 9 25M9 21V25M1 5C1 3 2.99999 1 5 1M1 5C1 7 1 15 1 17M1 5V17M5 1C7.00001 1 27 1 29 1M5 1H29M29 1C31 1 33 3 33 5M33 5C33 7 33 15 33 17M33 5V17M33 17C33 19 31 21 29 21M5 21C3 21 1 19 1 17M9 25C9 27 11 27 13 25" stroke="black" stroke-width="2" stroke-linejoin="round"/>
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