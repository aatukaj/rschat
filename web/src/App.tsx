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

function App() {


  const [messages, setMessages] = useState(Array<Message>);


  const { sendMessage, lastMessage, readyState } = useWebSocket('ws://127.0.0.1:8080');

  useEffect(() => {
    if (readyState === ReadyState.OPEN) {
      sendMessage(JSON.stringify(startmsg))
    }
  }, [readyState, sendMessage]);

  useEffect(() => {
    if (lastMessage !== null) {
      let data: Blob = lastMessage.data;
      data.text().then((text) => setMessages((prev) => prev.concat(JSON.parse(text))));
    }
  }, [lastMessage, setMessages]);


  return (
    <div className="App">
      <header className="App-header">
        <ul>
          {messages.map((msg, i) => <li key={i}><span style={{color: msg.color}}>{msg.user_name}:</span> {msg.content}</li>)}
        </ul>

      </header>
    </div>
  );
}

export default App;
