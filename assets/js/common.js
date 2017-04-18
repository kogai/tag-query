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

class Button extends React.Component {
  componentWillUnmount() {
    this.clipboard && this.clipboard.destroy();
  }

  componentDidMount() {
    this.clipboard = new Clipboard(this.refs.element);
  }

  render() {
    const {label, copy} = this.props;
    return (React.createElement("button", {
        className: "uk-button uk-button-primary",
        style: { margin: 4 },
        "data-clipboard-text": copy,
        ref: "element",
      }, label));
  } 
}

const Buttons = (htmlString, markdownString) => React.createElement("div", {
  className: "uk-margin",
}, [
  React.createElement(Button, {label: "Copy as HTML", copy: htmlString}),
  React.createElement(Button, {label: "Copy as Markdown", copy: markdownString}),
]);

const Root = (state) => {
  return React.createElement(
    'div',
    null,
    [
      Search("username", state.onChangeUsername),
      state.user_ids.length > 0 && Search("hashtag", state.onChangeHashtag),
      state.medias.map(m => React.createElement("img", {src: m.src})),
      Buttons(state.htmlString, state.markdownString),
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
        .then(xs => {
          const medias = xs.map(media => ({
            src: media.images.thumbnail.url,
            link: media.link
          }));

          const htmlString = medias.map(m => `<img src="${m.src}" />`).join("\n");
          const markdownString = medias.map(m => `![](${m.src})`).join("\n");

          this.setState({ medias });
          this.setState({ htmlString });
          this.setState({ markdownString });
        })
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
      htmlString: "",
      markdownString: "",
    };
  }
  render() {
    return Root(this.state);
  }
}

ReactDOM.render(
  React.createElement(Container),
  document.getElementById("root")
);
