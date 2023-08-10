<template>
  <div class="h-screen font-roboto relative">
    <ProfileTopBar :isMe="route.params.id === 'me'" class="z-20" @menu-pressed="menuPressed"/>
    <div v-if="currentTab === 0">
      <Profile @edit-profile-pressed="toggleLoginEditor" :profile="profile" :isMe="route.params.id === 'me'" class="absolute left-0 top-0 ml-16 mt-32"/>
      <div class="flex flex-col items-center h-full">
        <div class="flex justify-center items-center mt-4 w-[37rem] flex-col z-10">
          <SearchBar />
          <div class="flex space-x-8 justify-center mt-4">
            <Button text="Trending" class="bg-white hover:bg-neutral-200 w-44"/>
            <Button text="New" class="bg-white hover:bg-neutral-200 w-44"/>
          </div>
        </div>
        <div class="overflow-y-auto flex-grow">
          <Voxels/>
        </div>
      </div>
      <ProfileEditor :profile="profile" @close="toggleLoginEditor" v-if="showLoginEditor" class="absolute right-0 top-0 mr-16 mt-32"/>
    </div>
    <div v-if="currentTab === 1">
      <div class="flex flex-col items-center h-full justify-center">
        <div class="bg-black w-[33rem] lg:w-[65rem] 2xl:w-[90rem] h-0.5 mt-4"></div>
        <div class="overflow-y-auto flex-grow max-h-[calc(100vh-5.125rem)]">
          <div class="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-4 gap-8 p-4">
            <UserVoxel v-for="voxel in voxels" :voxel_id="voxel.voxel_id" :key="voxel.voxel_id" :name="voxel.name" />
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

function menuPressed(menu){
  currentTab.value = menu;
  console.log(menu);
}

function toggleLoginEditor() {
  showLoginEditor.value = !showLoginEditor.value;
}

let profile = ref(null);

onMounted(async () => {
  await getProfile();
  await getVoxels();
});

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
    let data = await res.json();
    console.log(data);
    profile.value = data;
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
    let data = await res.json();
    console.log(data);
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
    console.log(data);
  } else {
    let error = await res.text();
    console.error(error);
  }
}
</script>