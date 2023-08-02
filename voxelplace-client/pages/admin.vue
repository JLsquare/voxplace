<template>
  <div class="flex flex-col justify-center items-center font-roboto">
    <AdminTopBar
        :isAuth="isAuth"
        @voxels-pressed="currentTab = 0"
        @places-pressed="currentTab = 1"
        @users-pressed="currentTab = 2"
        @login-clicked="showLogin = true"
        @logout-clicked="isAuth = false"
        class="z-10"
    />
    <div class="mx-16 w-full flex flex-col items-center h-full">
      <div class="w-screen h-screen absolute top-0 pt-[10.5rem]">
        <div v-if="currentTab === 0"/>
        <AdminPlaces v-if="currentTab === 1"/>
        <div v-if="currentTab === 2"/>
      </div>
    </div>
    <div class="w-[28rem] z-10 mt-16">
      <AdminLogIn v-if="showLogin && !isAuth" @close-clicked="showLogin = false" @logged-in="isAuth = true"/>
    </div>
  </div>
</template>

<script setup>
import jwtDecode from "jwt-decode";

let currentTab = ref(0);
let isAuth = ref(false);
let showLogin = ref(false);

onMounted(() => {
  testToken();
});

async function testToken() {
  const token = localStorage.getItem('admin-token');

  if (token) {
    const decodedToken = jwtDecode(token);
    const dateNow = new Date();

    if (decodedToken.exp * 1000 < dateNow.getTime()) {
      console.log('Token expired.');
    } else {
      const response = await fetch('http://localhost:8000/api/user/checkadmin', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': localStorage.getItem('token')
        },
      });

      if (!response.ok) {
        const message = await response.text();
        console.log(message);
      } else {
        isAuth.value = await response.json();
      }
    }
  }
}
</script>