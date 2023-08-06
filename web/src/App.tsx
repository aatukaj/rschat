import React, { useEffect, useState } from 'react';
import useWebSocket, { ReadyState } from "react-use-websocket";

import './App.css';

// https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
enum Color {
  Black = "Black",
  Red = "Red",
  Green = "Green",
  Yellow = "Yellow",
  Blue = "Blue",
  Magenta = "Magenta",
  Cyan = "Cyan",
  Gray = "Gray",
  DarkGray = "DarkGray",
  LightRed = "LightRed",
  LightGreen = "LightGreen",
  LightYellow = "LightYellow",
  LightBlue = "LightBlue",
  LightMagenta = "LightMagenta",
  LightCyan = "LightCyan",
  White = "White",
}
enum AppState {
  Login,
  Chat,
}


interface NewUserSet {
  user_name: string,
  color: Color,
}
interface Message {
  user_name: String,
  content: String,
  color: Color,
}

const startmsg: NewUserSet = {
  user_name: "bob",
  color: Color.Blue,
}

function MessageList(props: { messages: Message[] }) {
  return (
    <ul>
      {props.messages.map((msg, i) => <li key={i}><span style={{ color: msg.color }}>{msg.user_name}:</span> {msg.content}</li>)}
    </ul>
  )
}

function App() {
  const [appState, setAppState] = useState(AppState.Login);

  const [messagesHistory, setMessagesHistory] = useState(Array<Message>);

  const [curMessage, setCurMessage] = useState("");

  const { sendMessage, lastMessage, readyState } = useWebSocket('ws://127.0.0.1:8080');

  useEffect(() => {
    if (readyState === ReadyState.OPEN) {
      sendMessage(JSON.stringify(startmsg))
    }
  }, [readyState, sendMessage]);

  useEffect(() => {
    if (lastMessage !== null) {
      let data: Blob = lastMessage.data;
      data.text().then((text) => setMessagesHistory((prev) => prev.concat(JSON.parse(text))));
    }
  }, [lastMessage, setMessagesHistory]);


  return (
    <div className="App">

      <MessageList messages={messagesHistory} />
      
      <form onSubmit={(e) => {
        e.preventDefault();
        console.log("IT WORKS")
        sendMessage(curMessage)
        setCurMessage("");
      }}>
        <input type="text" value={curMessage} onChange={(e) => setCurMessage(e.target.value)}></input>
        <input type="submit" hidden />
      </form>


    </div>
  );
}

export default App;


