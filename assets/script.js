const tableClass = {
    table: 'table table-striped',
}

function setLoading(show) {
    const loading = document.getElementById("loading")
    if (show) {
        loading.style.display = 'block'
    } else {
        loading.style.display = 'none'
    }
}

function getData(url) {
    setLoading(true)
    axios.get(url).then(function (response) {
        const data = response.data
        const columns = Object.keys(data[0])
        const tableData = data.map(function (row) {
            const d = columns.map(function (key) {
                return row[key]
            })
            return d
        })
        const newTable = new gridjs.Grid({
            columns,
            data: tableData,
            className: tableClass
        })
        const wrapper = document.getElementById("wrapper");
        while (wrapper.firstChild) {
            wrapper.removeChild(wrapper.firstChild)
        }
        newTable.render(wrapper)
        setLoading(false)
    })
}