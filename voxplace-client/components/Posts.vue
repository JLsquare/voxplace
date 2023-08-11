<template>
  <div class="flex flex-col h-full items-center">
    <div class="bg-black w-[40rem] border border-black mt-4"></div>
    <div class="overflow-y-scroll flex-grow scrollbar-hide grid grid-cols-1 gap-8 py-4">
      <Post
          v-for="post in posts"
          :post="post"
      />
    </div>
  </div>
</template>

<script setup>
let posts = ref([]);

const props = defineProps({
  user_id: {
    type: String,
    default: "0"
  },
  limit: {
    type: Number,
    default: 10
  }
})

onMounted(async () => {
  await getPosts();
});

async function getPosts(){
  const token = localStorage.getItem('token');

  const response = await fetch(`http://${window.location.hostname}:8000/api/post/top/${props.user_id}/${props.limit}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    }
  });
  if(response.ok){
    posts.value = await response.json();
    console.log(posts.value);
  } else {
    console.log(await response.text());
  }
}
</script>