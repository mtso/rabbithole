import { useEffect, useState } from "react";
import { useSelector, useDispatch } from "react-redux";
import { setId, setStatus } from "./rabbitSlice";

import logo from './logo.svg';
import './App.css';

import RabbitContainer from "./RabbitContainer";

function parseQuery(queryString) {
    var query = {};
    var pairs = (queryString[0] === '?' ? queryString.substr(1) : queryString).split('&');
    for (var i = 0; i < pairs.length; i++) {
        var pair = pairs[i].split('=');
        query[decodeURIComponent(pair[0])] = decodeURIComponent(pair[1] || '');
    }
    return query;
}

function App() {
  const [clicked, setClicked] = useState(false);
  const rabbit = useSelector((state) => state.rabbit);
  const { id } = rabbit;

  const dispatch = useDispatch();

  useEffect(() => {
    if (id === null) {
      let qs = parseQuery(window.location.search);
      console.log(qs);
      if (qs["rabbit"]) dispatch(setId({ rabbitId: qs["rabbit"] }));
    }
  }, [id, dispatch]);

  console.log(id);
  if (!id) {
    return (
      <div className="App">
        <button
          className="action_button"
          disabled={clicked}
          onClick={() => {
            setClicked(true);
            fetch("/api3/rabbits", {
              method: "POST",
              body: JSON.stringify({ name: "hello" }),
            }).then((resp) => resp.json()).then((rabbit) => {
              let p = new URLSearchParams();
              p.set("rabbit", rabbit.id);
              window.location.search = p;
            }).catch(console.error);
          }}
        >Create Rabbit</button>
      </div>
    );
  }
  else {
    return <RabbitContainer />
  }

/*
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>
          Edit <code>src/App.js</code> and save to reload.
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );*/
}

export default App;
