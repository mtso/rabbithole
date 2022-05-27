import RabbitSvg from "./RabbitSvg";

export default function Rabbit({ id, status, bodyColor, patchColor, eyeColor }) {
  return (
    <div className="App">
     <RabbitSvg
       bodyColor={bodyColor}
       patchColor={patchColor}
       eyeColor={eyeColor}
     />
    </div>
  )
}

