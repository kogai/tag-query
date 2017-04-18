const Search = (name, onChange) => React.createElement("div", {
  className: 'uk-margin'
}, [
  React.createElement("label", {className: "uk-form-label"}, name),
  React.createElement("div", {className: "uk-form-controls"}, React.createElement("input", {
    className: "uk-input",
    type: "text",
    placeholder: `input ${name}`,
    onChange: e => onChange(e.target.value),
  })),
]);

const Root = (state) => {
  return React.createElement(
    'div',
    null,
    [
      Search("username", state.onChangeUsername),
      state.user_ids.length > 0 && Search("hashtag", state.onChangeHashtag),
      state.medias.map(m => React.createElement("img", {src: m.src}))
    ]
  );
};

class Container extends React.Component {
  constructor() {
    super();
    const onChangeUsername = username => {
      fetch(`/api/username?username=${username}`, {
          credentials: 'include'
        })
        .then(r => r.json())
        .then(r => r.data && this.setState({ user_ids: r.data.map(d => d.id) }))
        ;

      this.setState({ username });
    };
    const onChangeHashtag = hashtag => {
      const user_ids = this.state.user_ids;
      fetch(`/api/hashtag?user_id=${user_ids[0]}&hashtag=${hashtag}`, {
          credentials: 'include'
        })
        .then(r => r.json())
        .then(xs => this.setState({ medias: xs.map(media => ({ src: media.images.thumbnail.url, link: media.link })) }))
        ;

      this.setState({ hashtag });
    };

    this.state = {
      username: "",
      hashtag: "",
      user_ids: [],
      medias: [],
      onChangeUsername,
      onChangeHashtag,
    };
  }
  render() {
    console.log(this.state);
    return Root(this.state);
  }
}

ReactDOM.render(
  React.createElement(Container),
  document.getElementById("root")
);
