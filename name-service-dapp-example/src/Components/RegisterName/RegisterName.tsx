import { FunctionComponent, useState } from "react";
import { SubHeader } from 'Components/Headers';
import { Button, Input } from 'Components';
import styled from "styled-components";

interface RegisterNameProps {
    onRegister: (s: string) => Promise<any>
}

export const RegisterName: FunctionComponent<RegisterNameProps> = ({ onRegister }) => {
    const [newName, setNewName] = useState("");
    const [registering, setRegistering] = useState(false)
    const nameValid = newName.trim() != ''

    const handleRegistration = () => {
        setRegistering(true)
        onRegister(newName).then(() => {
            setNewName('')
        }).finally(() => {
            setRegistering(false)
        })
    }

    return <RegisterWrapper>
        <SubHeader>Register a new name</SubHeader>
        <form>
            <Input disabled={registering} value={newName} onChange={(n) => setNewName(n)} />
            <Button disabled={registering || !nameValid} type="submit" onClick={handleRegistration}>Register</Button>
        </form>
    </RegisterWrapper>
}

const RegisterWrapper = styled.div`
    margin-bottom: 20px;
`