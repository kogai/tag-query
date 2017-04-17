const hashtag = document.querySelector("#hashtag");
const username = document.querySelector("#username");

hashtag.addEventListener("input", (e) => {
  const value = e.target.value;
  fetch(`/api/hashtag?user_id=31480262&hashtag=${value}`, {
      credentials: 'include'
    })
    .then(r => r.json())
    .then(r => console.log(r));
});

username.addEventListener("input", (e) => {
  const value = e.target.value;
  fetch(`/api/username?username=${value}`, {
      credentials: 'include'
    })
    .then(r => r.json())
    .then(r => console.log(r));
});