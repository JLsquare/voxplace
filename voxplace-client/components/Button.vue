<template>
  <NuxtLink :to="routerLink" class="rounded-2xl p-0 m-0 bg-transparent hover:bg-transparent">
    <div :class="['border-2 border-black rounded-2xl justify-center items-center px-4 shadow-custom w-44 text-center', disabled ? 'border-neutral-500 bg-neutral-100 hover:bg-neutral-100' : ['cursor-pointer', $attrs.class]]">
      <p :class="['text-lg', disabled ? 'text-neutral-500' : 'text-black']">{{ text }}</p>
    </div>
  </NuxtLink>
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
  },
  routerLink: {
    type: String,
    default: ''
  }
})
</script>
