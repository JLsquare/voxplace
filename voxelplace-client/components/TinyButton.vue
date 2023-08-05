<template>
  <div :class="['border-2 border-black rounded-2xl justify-center items-center h-fit px-6 text-center', disabled ? 'border-neutral-500 bg-neutral-100 hover:bg-neutral-100' : 'cursor-pointer', $attrs.class]">
    <p :class="[disabled ? 'text-neutral-500' : 'text-black']">{{ text }}</p>
  </div>
</template>

<script setup>
const props = defineProps({
  text: {
    type: String,
    required: true
  },
  disabled: {
    type: Boolean,
    default: false
  }
})
</script>
