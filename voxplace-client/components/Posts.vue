<template>
  <div class="flex flex-col h-full items-center">
    <div class="bg-black w-[40rem] border border-black mt-4"></div>
    <div class="overflow-y-scroll flex-grow scrollbar-hide grid grid-cols-1 gap-8 py-4">
      <Post
          v-for="post in posts"
          :post="post"
          @open-post="openPost"
      />
    </div>
    <div v-if="postOpen" class="w-screen h-screen bg-black absolute top-0 opacity-20 z-10"></div>
    <OpenedPost :post_id="selectedPost.post_id" v-if="postOpen" @close-post="postOpen = false" class="z-10 absolute top-24"/>
  </div>
</template>

<script setup>
let posts = ref([]);
let selectedPost = ref(null);
let postOpen = ref(false);

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

function openPost(post){
  selectedPost.value = post;
  postOpen.value = true;
}

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