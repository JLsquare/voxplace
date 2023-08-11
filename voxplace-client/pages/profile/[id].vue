<template>
  <div class="h-screen font-roboto">
    <ProfileTopBar :isMe="route.params.id === 'me'" @edit-profile-pressed="toggleLoginEditor" @menu-pressed="menuPressed"/>
    <div style="height: calc(100% - 10.25rem);">
      <div v-if="currentTab === 0" class="h-full flex justify-center space-x-16 w-full">
        <Profile @edit-profile-pressed="toggleLoginEditor" v-if="profile" :profile="profile" :isMe="route.params.id === 'me'" class="mt-4"/>
        <div class="flex flex-col items-center h-full">
          <div class="flex justify-center items-center mt-4 flex-col w-full">
            <SearchBar class="w-[37rem]"/>
            <div class="flex space-x-8 justify-center mt-4">
              <Button text="Trending" class="bg-white hover:bg-neutral-200 w-44"/>
              <Button text="New" class="bg-white hover:bg-neutral-200 w-44"/>
            </div>
          </div>
          <Posts :user_id="route.params.id"/>
        </div>
        <ProfileEditor :profile="profile" @close="toggleLoginEditor" v-if="profile" :class="['mt-4', showLoginEditor ? '' : 'invisible']"/>
      </div>
      <div v-if="currentTab === 1" class="h-full flex flex-col justify-center items-center">
        <div class="flex flex-col items-center h-full justify-center">
          <div class="bg-black w-[33rem] lg:w-[65rem] 2xl:w-[90rem] h-0.5 mt-4"></div>
          <div class="overflow-y-auto flex-grow max-h-[calc(100vh-5.125rem)]">
            <div class="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-4 gap-8 p-4">
              <UserVoxel v-for="voxel in voxels" :voxel_id="voxel.voxel_id" :key="voxel.voxel_id" :name="voxel.name" @click="openVoxelEditor(voxel)" />
              <div class=" py-4 px-16 bg-white border-2 border-black rounded-2xl w-[20rem] h-[20rem] flex flex-col justify-between items-center shadow-custom relative order-first">
                <div class="flex flex-col items-center">
                  <p class="text-xl">New Voxel</p>
                  <div class="bg-black w-full h-0.5 mt-1"></div>
                </div>
                <p class="mt-2">Name :</p>
                <input v-model="newVoxelName" type="text" class="border-2 border-black rounded-lg w-full px-2"/>
                <p class="mt-2">Width :</p>
                <input v-model="newVoxelSize" type="number" class="border-2 border-black rounded-lg w-full px-2" min="4" max="512" step="4"/>
                <div class="flex justify-center mt-8">
                  <Button text="Create" class="bg-blue-300" @click="createVoxel"/>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-if="postOpen" class="z-10 border-2 border-black rounded-2xl absolute top-32 bg-white p-4 flex flex-col w-1/4 h-3/5 shadow-custom">
          <CloseButton class="absolute top-4 right-4" @click="postOpen = false" />
          <p class="text-lg">Title :</p>
          <input v-model="selectedVoxel.name" type="text" class="border-2 border-black rounded-lg w-2/3 px-2"/>
          <p class="text-lg mt-2">Voxel :</p>
          <div class="w-full h-full p-4 border-2 border-black rounded-2xl">
            <VoxelPreview :voxel_id="selectedVoxel.voxel_id" class="w-full h-full" />
          </div>
          <p class="mt-2 text-lg">Content :</p>
          <textarea v-model="selectedVoxel.content" type="text" class="border-2 border-black rounded-lg w-full px-2 h-32 resize-none"/>
          <div class="flex items-center justify-between w-full mt-4">
            <Button text="Cancel" class="bg-white hover:bg-neutral-300 w-28" @click="postOpen = false" />
            <Button text="Post" @click="postVoxel" class="bg-blue-300 hover:bg-blue-400 w-28"/>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import jwtDecode from "jwt-decode";

const route = useRoute();
let showLoginEditor = ref(false);
let currentTab = ref(0);
let newVoxelName = ref('New voxel');
let newVoxelSize = ref(4);
let voxels = ref([]);
let selectedVoxel = ref({voxel_id: '0', name: 'New voxel'});
let postOpen = ref(false);
let profile = ref(null);

onMounted(async () => {
  await getProfile();
  await getVoxels();
});

function openVoxelEditor(voxel) {
  selectedVoxel.value = voxel;
  postOpen.value = true;
}

function menuPressed(menu){
  currentTab.value = menu;
}

function toggleLoginEditor() {
  showLoginEditor.value = !showLoginEditor.value;
}

async function getProfile() {
  const token = localStorage.getItem('token');

  if(token) {
    const decodedToken = jwtDecode(token);

    if (decodedToken.sub === route.params.id) {
      route.params.id = 'me';
    }
  }

  let res = await fetch(`http://${window.location.hostname}:8000/api/user/profile/${route.params.id}`, {
    headers: {
      'Authorization': token
    }
  });
  if(res.ok) {
    profile.value = await res.json();
  } else {
    let error = await res.text();
    console.error(error);
  }
}

async function createVoxel(){
  const token = localStorage.getItem('token');

  let res = await fetch(`http://${window.location.hostname}:8000/api/user/voxels/create`, {
    method: 'POST',
    headers: {
      'Authorization': token,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name: newVoxelName.value,
      size: [newVoxelSize.value, newVoxelSize.value, newVoxelSize.value]
    })
  });

  if(res.ok) {
    await getVoxels();
  } else {
    let error = await res.text();
    console.error(error);
  }
}

async function getVoxels(){
  const token = localStorage.getItem('token');

  let res = await fetch(`http://${window.location.hostname}:8000/api/user/voxels/all`, {
    headers: {
      'Authorization': token
    }
  });

  if(res.ok) {
    let data = await res.json();
    let profileVoxel = profile.value;
    profileVoxel.name = profileVoxel.username + "'s Profile";
    data[data.length] = profileVoxel;
    voxels.value = data;
  } else {
    let error = await res.text();
    console.error(error);
  }
}

async function postVoxel() {
  const token = localStorage.getItem('token');

  let res = await fetch(`http://${window.location.hostname}:8000/api/post/create`, {
    method: 'POST',
    headers: {
      'Authorization': token,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      title: selectedVoxel.value.name,
      voxel_id: selectedVoxel.value.voxel_id,
      content: selectedVoxel.value.content
    })
  });

  if(res.ok) {
    postOpen.value = false;
  } else {
    let error = await res.text();
    console.error(error);
  }
}
</script>