<template>
  <div class="font-roboto h-screen">
    <IndexTopBar :auth-button-index="authId" :is-auth="isAuth" @login-clicked="handleLoginClicked" @posts-pressed="setTab(0)" @places-pressed="setTab(1)" @users-pressed="setTab(2)" />
    <div style="height: calc(100% - 10.25rem);">
      <div class="flex justify-center items-center mt-4 flex-col w-full">
        <SearchBar class="w-[37rem]"/>
        <div class="flex space-x-8 justify-center mt-4">
          <Button text="Top" class="bg-white hover:bg-neutral-200 w-44"/>
          <Button text="Following" disabled class="w-44"/>
          <Button text="New" class="bg-white hover:bg-neutral-200 w-44"/>
        </div>
      </div>
      <div class="h-full">
        <Posts v-if="currentTab === 0"/>
        <Places v-if="currentTab === 1"/>
        <Users v-if="currentTab === 2"/>
      </div>
      <div v-if="(showLogin || showSignUp) && !isAuth" class="w-screen h-screen bg-black absolute top-0 opacity-20 z-10"></div>
    </div>
    <div class="w-[28rem] z-10">
      <LogIn v-if="showLogin && !isAuth" @signup-clicked="showSignUp = true; showLogin = false; authId = 1" @close-clicked="showLogin = false" @logged-in="isAuth = true"/>
      <SignUp v-if="showSignUp && !isAuth" @login-clicked="showLogin = true; showSignUp = false; authId = 0" @close-clicked="showSignUp = false" @signed-up="isAuth = true"/>
    </div>
  </div>
</template>

<script setup>
import jwtDecode from "jwt-decode";

let showLogin = ref(false);
let showSignUp = ref(false);
let authId = ref(0);
let currentTab = ref(0);
let isAuth = ref(false);

onMounted(() => {
  testToken();
});

function testToken() {
  const token = localStorage.getItem('token');

  if(token) {
    const decodedToken = jwtDecode(token);
    const dateNow = new Date();

    if(decodedToken.exp * 1000 < dateNow.getTime()){
      console.log('Token expired.');
    } else {
      isAuth.value = true;
    }
  }
}

function handleLoginClicked() {
  if(showLogin.value || showSignUp.value) {
    showLogin.value = false;
    showSignUp.value = false;
  } else {
    if(authId.value === 0) {
      showLogin.value = true;
    } else {
      showSignUp.value = true;
    }
  }
}

const setTab = (index) => {
  currentTab.value = index;
  console.log(currentTab.value);
};
</script>

<style>
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>