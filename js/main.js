async function convertToHangul() {
    const inputText = document.getElementById('inputText').value;

    try {
        // 서버로 POST 요청 보내기
        const response = await fetch('/convert', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ text: inputText }),
        });

        // 서버 응답 처리
        if (!response.ok) {
            throw new Error('서버 요청에 실패했습니다.');
        }

        const data = await response.json();

        // 변환된 문자열을 outputText에 출력
        document.getElementById('outputText').value = data.converted_text || '변환 실패';
    } catch (error) {
        console.error('오류 발생:', error);
        document.getElementById('outputText').value = '오류 발생: 변환에 실패했습니다.';
    }
}