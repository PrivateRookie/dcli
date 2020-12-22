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

const DataGrid = new gridjs.Grid({
  columns: [],
  data: []
})
const wrapper = document.getElementById("wrapper")
DataGrid.render(wrapper)

function fetchData(url) {
  setLoading(true)
  // const wrapper = document.getElementById("wrapper")
  // while (wrapper.firstChild) {
  //   wrapper.removeChild(wrapper.firstChild)
  // }
  axios.get(url).then(function (response) {
    const data = response.data
    const columns = Object.keys(data[0])
    DataGrid.updateConfig({
      columns,
      data,
      // className: tableClass
    }).forceRender()
    setLoading(false)
  })
}

function fetchMeta() {
  axios.get('api/_meta').then(resp => {
    const { prefix } = resp.data
    const { protocol, host } = window.location
    const apis = resp.data.queries.map(query => {
      const isMeta = query.url.endsWith('_meta')
      const url = `${protocol}//${host}/${prefix}/${query.url}`
      const sql = gridjs.html(`<code>${query.sql}</code>`)
      const urlCell = gridjs.html(`
        <div class="alert alert-light role="alert">
          <a class="alert-link" href="${url}">${url}</a>
        </div>
      `)
      if (isMeta) {
        const buttonCell = gridjs.html(`<button type="button" class="btn btn-outline-info" disabled>Try It</button>`)
        return { ...query, url, urlCell, buttonCell }
      } else {
        const buttonCell = gridjs.html(`<button type="button" class="btn btn-outline-info" onclick="fetchData('${url}')">Try It</button>`)
        return { ...query, sql, url, urlCell, buttonCell }

      }
    })

    const newTable = new gridjs.Grid({
      columns: ['name', 'profile', 'sql', 'urlCell', 'buttonCell'],
      data: apis
    })

    const ele = document.getElementById('meta')
    while (ele.firstChild) {
      ele.removeChild(ele.firstChild)
    }
    newTable.render(ele)
  })
}