import React, { useEffect, useState } from 'react';
import useWebSocket, { ReadyState } from "react-use-websocket";



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
      {props.messages.map((msg, i) => <li key={i}>
        <span style={{ color: msg.color, }}><b>{msg.user_name}:</b></span> {msg.content}
      </li>)}
    </ul>
  )
}

const colorToClass = {
  [Color.Black]: "bg-zinc-950",
  [Color.Red]: "bg-red-600",
  [Color.Green]: "bg-green-500",
  [Color.Yellow]: "bg-yellow-400",
  [Color.Blue]: "bg-blue-600",
  [Color.Magenta]: "bg-fuchsia-600",
  [Color.Cyan]: "bg-cyan-500",
  [Color.Gray]: "bg-zinc-500",
  [Color.DarkGray]: "bg-zinc-900",
  [Color.LightRed]: "bg-red-400",
  [Color.LightGreen]: "bg-green-300",
  [Color.LightYellow]: "bg-yellow-200",
  [Color.LightBlue]: "bg-blue-300",
  [Color.LightMagenta]: "bg-fuchsia-300",
  [Color.LightCyan]: "bg-cyan-200",
  [Color.White]: "bg-white",
}


function Login() {
  const [userName, setUserName] = useState("");
  const [userColor, setUserColor] = useState(Color.White);

  return (
    <><h1 className="text-3xl font-bold underline">
      log in wää
    </h1>
      <h3>Preview: {userName} {userColor}</h3>
      <form className="bg-zinc-900 rounded-lg p-5 shadow-xl" onSubmit={(e) => e.preventDefault()}>
        <input
          type="text"
          className="rounded-md bg-zinc-800 p-1 shadow-xl placeholder:italic placeholder:text-zinc-600"
          placeholder="Username" value={userName} maxLength={15} onChange={(e) => setUserName(e.target.value)}>
        </input><br />
        <div className="grid grid-cols-4 items-stretch gap-x-1 gap-y-1 mt-2">
          {(Object.keys(Color) as Array<Color>).map((color, i) =>
            <input
              key={i} type="radio"
              className={`appearance-none rounded-md border-2 border-zinc-800 h-8 m-0 ring-transparent ${colorToClass[color]} ring-1 focus:ring-white`}
              value={color} onChange={(e => setUserColor(e.target.value as Color))} name="color"
            />
          )}
        </div>
      </form></>
  );
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

    <div className="flex flex-col justify-center h-screen items-center">
      {
        appState === AppState.Login ? <Login /> :
          <>
            <MessageList messages={messagesHistory} />

            <form onSubmit={(e) => {
              e.preventDefault();
              sendMessage(curMessage)
              setCurMessage("");
            }}>
              <input type="text" value={curMessage} onChange={(e) => setCurMessage(e.target.value)}></input>
              <input type="submit" hidden />
            </form>


          </>
      }

    </div>

  );
}

export default App;


