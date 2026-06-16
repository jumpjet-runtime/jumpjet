import { log } from 'jumpjet:runtime/debug'
import * as gpu from 'jumpjet:runtime/gpu'
import * as window from 'jumpjet:runtime/window'

import * as THREE from 'three'
import { JumpjetRenderer } from '@jumpjet-runtime/threejs-renderer'

let camera, renderer, scene, mesh

export const guest = {
  init() {
    const [width, height] = window.dimensions()

    camera = new THREE.PerspectiveCamera(70, width / height, 0.01, 10)
    camera.position.z = 1

    scene = new THREE.Scene()

    const geometry = new THREE.BoxGeometry(0.2, 0.2, 0.2)
    const material = new THREE.MeshNormalMaterial()

    mesh = new THREE.Mesh(geometry, material)
    scene.add(mesh)

    renderer = new JumpjetRenderer(gpu)
    renderer.setSize(width, height)
  },
  update(time, deltaTime) {
    log('update')
  },
  render(time, deltaTime) {
    mesh.rotation.x = time / 2000
    mesh.rotation.y = time / 1000

    renderer.render(scene, camera)
  }
}
