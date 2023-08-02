<template>
  <div class="bg-white border-2 border-black rounded-2xl w-[28rem] flex flex-col items-center p-8 shadow-custom relative">
    <CloseButton class="absolute top-4 right-4" @click="$emit('close-clicked')" />
    <p class="text-3xl">Log In</p>
    <div class="bg-black w-3/5 h-0.5 mt-2"></div>
    <div class="bg-white border-2 border-black rounded-2xl w-96 mt-8 flex justify-between items-center px-4 shadow-custom">
      <input v-model="username" type="text" placeholder="Username" class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none" />
    </div>
    <div class="mt-2 h-6 flex items-start w-full">
      <p v-if="emptyUsername" class="text-red-500 text-sm">Username must not be empty.</p>
      <p v-if="invalidCredentials" class="text-red-500 text-sm">Username or password is incorrect.</p>
    </div>
    <div class="bg-white border-2 border-black rounded-2xl w-96 mt-2 flex justify-between items-center px-4 shadow-custom">
      <input v-model="password" type="password" placeholder="Password" class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none" />
    </div>
    <div class="mt-2 h-6 flex items-start w-full">
      <p v-if="emptyPassword" class="text-red-500 text-sm">Password must not be empty.</p>
      <p v-if="invalidCredentials" class="text-red-500 text-sm">Username or password is incorrect.</p>
    </div>
    <div class="flex items-start mt-4 w-full">
      <p>Forgot your</p>
      <p class="text-blue-400 underline ml-2 cursor-pointer">username</p>
      <p class="ml-2">or</p>
      <p class="text-blue-400 underline ml-2 cursor-pointer">password</p>
      <p>?</p>
    </div>
    <div class="mt-8">
      <Button text="Log In" @click="login" class="bg-blue-300 hover:bg-blue-400"/>
    </div>
    <div class="flex mt-8 items-start w-full">
      <p>Don't have an account?</p>
      <p @click="$emit('signup-clicked')" class="underline text-blue-400 ml-2 cursor-pointer">Sign Up</p>
    </div>
  </div>
</template>

<script setup>
let username = ref('');
let password = ref('');
let token = ref('');

let emptyUsername = ref(false);
let emptyPassword = ref(false);
let invalidCredentials = ref(false);

const emit = defineEmits(['close-clicked', 'signup-clicked', 'logged-in']);

async function login() {
  emptyUsername.value = false;
  emptyPassword.value = false;
  invalidCredentials.value = false;

  emptyUsername.value = username.value.trim().length === 0;
  emptyPassword.value = password.value.trim().length === 0;

  if (emptyUsername.value || emptyPassword.value) {
    return;
  }

  let user = {
    username: username.value,
    password: password.value,
  }

  const response = await fetch('http://localhost:8000/api/user/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(user)
  });

  if (!response.ok) {
    const message = await response.text();
    console.log(message);
    invalidCredentials.value = true;
  }

  token.value = await response.json();

  localStorage.setItem('token', token.value);

  emit('logged-in');
}
</script>
