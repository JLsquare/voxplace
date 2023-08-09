<template>
  <div class="h-screen font-roboto">
    <ProfileTopBar/>
    <Profile @edit-profile-pressed="toggleLoginEditor" :profile="profile" :isMe="route.params.id === 'me'" class="absolute left-0 top-0 ml-16 mt-32"/>
    <div class="flex flex-col items-center">
      <div class="flex justify-center items-center mt-4 w-[37rem] flex-col z-10">
        <SearchBar />
        <div class="flex space-x-8 justify-center mt-4">
          <Button text="Trending" class="bg-white hover:bg-neutral-200 w-44"/>
          <Button text="New" class="bg-white hover:bg-neutral-200 w-44"/>
        </div>
      </div>
      <div class="h-screen absolute top-0 pt-[10.5rem]">
        <Voxels/>
      </div>
    </div>
    <ProfileEditor :profile="profile" @close="toggleLoginEditor" v-if="showLoginEditor" class="absolute right-0 top-0 mr-16 mt-32"/>
  </div>
</template>

<script setup>
const route = useRoute();
let showLoginEditor = ref(false);

function toggleLoginEditor() {
  showLoginEditor.value = !showLoginEditor.value;
}

let profile = ref(null);

onMounted(() => {
  getProfile();
});

async function getProfile() {
  const token = localStorage.getItem('token');

  let res = await fetch(`http://${window.location.hostname}:8000/api/user/profile/${route.params.id}`, {
    headers: {
      'Authorization': token
    }
  });
  if(res.ok) {
    let data = await res.json();
    console.log(data);
    profile.value = data;
  } else {
    let error = await res.text();
    console.error(error);
  }
}
</script>