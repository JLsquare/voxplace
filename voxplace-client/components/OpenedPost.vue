<template>
  <div v-if="post" class="border-2 border-black rounded-2xl bg-white w-2/5 h-4/5 flex flex-col shadow-custom">
    <CloseButton class="absolute top-4 right-4" @click="$emit('close-post')" />
    <p class="text-xl ml-4 mt-4">Username</p>
    <p class="text-2xl ml-4">{{post.title}}</p>
    <div class="bg-black border border-black mx-4"></div>
    <div class="overflow-y-scroll scrollbar-hide flex flex-col flex-grow">
      <div class="h-3/4 p-4 mx-4 border-2 border-black rounded-2xl mt-2">
        <VoxelPreview :voxel_id="post.voxel_id" class="w-full h-full" />
      </div>
      <div class="flex p-2 mx-4">
        <p class="w-full text-xl">{{ post.content }}</p>
        <div class="items-center flex flex-col justify-center">
          <ArrowButton class=""/>
          <p class="text-xl">0</p>
          <ArrowButton class="transform rotate-180"/>
        </div>
      </div>
      <div class="bg-black border border-black mx-4"></div>
      <div class="bg-white border-2 border-black rounded-2xl flex justify-between items-center px-4 mt-4 shadow-custom mx-4">
        <input
            type="text"
            v-model="newComment"
            placeholder="New comment"
            class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none"
        />
        <SendButton class="ml-2" @click="sendComment" size="24"/>
      </div>
      <div class="mt-2 pb-2 mx-4">
        <Comment
            v-for="comment in comments"
            :comment="comment"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
let post = ref(null);
let comments = ref([]);
let newComment = ref('');

const props = defineProps({
  post_id: {
    type: String,
    required: true
  }
})

onMounted(() => {
  getPost()
  getComments()
})

async function getPost() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/post/${props.post_id}`)

  if(response.ok){
    post.value = await response.json()
  } else {
    console.log(await response.text())
  }
}

async function getComments() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/comment/post/${props.post_id}`)

  if(response.ok){
    comments.value = await response.json()
    console.log(comments.value)
  } else {
    console.log(await response.text())
  }
}

async function sendComment() {
  let token = localStorage.getItem('token')

  const response = await fetch(`http://${window.location.hostname}:8000/api/comment/create`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify({
      post_id: props.post_id,
      content: newComment.value
    })
  })

  if(response.ok){
    const comment = await response.json()
    comments.value.push(comment)
    newComment.value = ''
  } else {
    console.log(await response.text())
  }
}
</script>