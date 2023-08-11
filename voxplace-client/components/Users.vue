<template>
  <div class="w-full flex flex-col h-full items-center">
    <div class="bg-black w-[45rem] lg:w-[67rem] 2xl:w-[89rem] h-0.5 mt-4"></div>
    <div class="overflow-y-scroll overflow-hidden block h-full scrollbar-hide">
      <div class="grid grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-8 p-4">
        <User
            v-for="user in users"
            :username="user.username"
            :user_id="user.user_id"
            :voxel_id="user.voxel_id"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
let users = ref([]);

onMounted(async () => {
  await getUsers();
});

async function getUsers(){
  const response = await fetch(`http://${window.location.hostname}:8000/api/user/top/10`);
  if(response.ok){
    users.value = await response.json();
    console.log(users.value);
  } else {
    console.log(await response.text());
  }
}
</script>
