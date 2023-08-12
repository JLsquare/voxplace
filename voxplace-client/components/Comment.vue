<template>
  <div class="flex p-2">
    <NuxtLink :to="`/user/${comment.user_id}`" class="text-lg">{{ comment.username }} : </NuxtLink>
    <p class="text-lg ml-2">{{ comment.content }}</p>
  </div>
</template>

<script setup>
const props = defineProps({
  comment: {
    type: Object,
    default: () => ({
      author_id: 0,
      username: "Author",
      content: "comment test",
    })
  }
})
</script>